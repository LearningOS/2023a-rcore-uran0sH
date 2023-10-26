# 地址空间
## 虚拟地址与地址空间
CPU 中的 内存管理单元 (MMU, Memory Management Unit) 自动将这个虚拟地址进行 地址转换 (Address Translation) 变为一个物理地址，即这个应用的数据/指令的物理内存位置。

## 分页内存管理
![page_table](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/page-table.png)
### SV39 多级页表的硬件机制
可以通过修改 S 特权级的一个名为 `satp` 的 CSR 来启用分页模式，在这之后 S 和 U 特权级的访存地址会被视为一个虚拟地址，它需要经过 MMU 的地址转换变为一个物理地址，再通过它来访问物理内存；而 M 特权级的访存地址，我们可设定是内存的物理地址。
![satp](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/satp.png)
1. MODE 控制 CPU 使用哪种页表实现，设置为8，SV39分页机制开启

2. ASID 表示地址空间标识符，这里还没有涉及到进程的概念，我们不需要管这个地方；

3. PPN 存的是根页表所在的物理页号。这样，给定一个虚拟页号，CPU 就可以从三级页表的根页表开始一步步的将其映射到一个物理页号。

地址格式与组成：
![sv39-va-pa](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/sv39-va-pa.png)

页表项的数据结构抽象与类型定义
![sv39-pte](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/sv39-pte.png)

SV39 地址转换过程：
![sv39-full](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/sv39-full.png)

TLB:

MMU中的 快表（TLB, Translation Lookaside Buffer） 来作为虚拟页号到物理页号的映射的页表缓存。内核要在修改 satp 的指令后面马上使用 sfence.vma 指令刷新清空整个 TLB。

在未开启分页之前，内核可以通过物理地址直接访问内存，但是开启分页后，内核的访存也会被视为一个虚拟地址，需要借助 MMU 才能将虚拟地址转换为物理地址。我们要在内核里访问特定的物理地址 `pa` 就需要构造一个 `va`，建立一个 `va->pa` 的映射。rCore 中使用恒等映射，即每个 `ppn` 都有一个对应的 `vpn`（还有其他方法，比如 Recursive Mapping）
```rust
// os/src/mm/address.rs

impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)
        }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            (pa.0 as *mut T).as_mut().unwrap()
        }
    }
}
```
开启分页之后的关系：va->pa(v)->pa

### 内核地址空间
启用分页模式下，内核代码的访存地址也会被视为一个虚拟地址并需要经过 MMU 的地址转换，因此我们也需要为内核对应构造一个地址空间，它除了仍然需要允许内核的各数据段能够被正常访问之后，还需要包含所有应用的内核栈以及一个 跳板 (Trampoline) 。
![kernel-as-high](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/kernel-as-high.png)

### 应用地址空间
![app-as-full](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/app-as-full.png)

## 基于地址空间的分时多任务
```rust
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
```
activate 完分页机制才会开启。

切换 satp CSR 是否是一个 平滑 的过渡：其含义是指，切换 satp 的指令及其下一条指令这两条相邻的指令的虚拟地址是相邻的（由于切换 satp 的指令并不是一条跳转指令， pc 只是简单的自增当前指令的字长），而它们所在的物理地址一般情况下也是相邻的，但是它们所经过的地址转换流程却是不同的——切换 satp 导致 MMU 查的多级页表是不同的。这就要求前后两个地址空间在切换 satp 的指令 附近 的映射满足某种意义上的连续性。