# lab1
## 实现过程
在 `TaskControlBlock` 中添加：

```rust
    /// The task's syscall times.
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
```

在 `TrapHandler` 处理系统调用时候添加

```rust
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            cx.sepc += 4;
            TASK_MANAGER.add_current_syscall_times(cx.x[17]);
            // get system call return value
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
    }
```

给 `TaskManager` 实现两个方法

```rust
    /// Calculate syscall times
    pub fn add_current_syscall_times(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[syscall_id] += 1;
    }

    /// Get the number of current task's syscall
    pub fn get_current_syscall_times(&self) -> [u32; MAX_SYSCALL_NUM] {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times.clone()
    }
```

实现

```rust
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    unsafe {
        (*ti).status = TaskStatus::Running;
        (*ti).time = get_time_ms();
        (*ti).syscall_times = TASK_MANAGER.get_current_syscall_times();
        return 0;
    }
}
```
## 简答作业
1. RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0. 
```bash
[ERROR] [kernel] .bss [0x8027c000, 0x802a5000)
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003c4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```
2. 
    1. TrapContext 的地址。a. 正常从__alltraps走下来的trap_handler流程 b. app第一次被__switch的时候通过__restore开始运行。
    2. sstatus(SPP 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息), sepc(当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址), sscratch
    3. x2 是 sp 会在后面保存，x4 是 tp 不被使用不需要保存
    4. sp 指向内核栈，sscratch 指向用户栈
    5. sret：CPU 会将当前的特权级按照 sstatus 的 SPP 字段设置为 U 或者 S ；U 会跳转到 sepc 寄存器指向的那条指令，然后继续执行。
    6. sp 指向内核栈，sscratch 指向用户栈
    7. `ecall`

## 荣誉准则
在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：


此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：


3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

