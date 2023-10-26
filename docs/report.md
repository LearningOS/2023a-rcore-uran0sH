# 多道程序与分时多任务

## 任务放置
将多个程序一次性加载入内存，需要将程序依次放置在内存的不同位置

每个程序有相应的内核栈和用户栈

## 任务切换

任务切换是来自两个不同应用在内核中的 Trap 控制流之间的切换。

关键在于 `__switch` 这个函数，负责栈的切换

![switch](https://rcore-os.cn/rCore-Tutorial-Book-v3/_images/switch.png)
上图表示从A切换到B
1. 在 Trap 控制流 A 调用 `__switch` 之前，A 的内核栈上只有 Trap 上下文和 Trap 处理函数的调用栈信息，而 B 是之前被切换出去的；
2. A 在 A 任务上下文空间在里面保存 CPU 当前的寄存器快照；
3. 这一步极为关键，读取 `next_task_cx_ptr` 指向的 B 任务上下文，根据 B 任务上下文保存的内容来恢复 `ra` 寄存器、`s0~s11` 寄存器以及 `sp` 寄存器。只有这一步做完后， `__switch` 才能做到一个函数跨两条控制流执行，即 通过换栈也就实现了控制流的切换 。
4. 上一步寄存器恢复完成后，可以看到通过恢复 `sp` 寄存器换到了任务 B 的内核栈上，进而实现了控制流的切换。这就是为什么 `__switch` 能做到一个函数跨两条控制流执行。此后，当 CPU 执行 `ret` 汇编伪指令完成 `__switch` 函数返回后，任务 B 可以从调用 `__switch` 的位置继续向下执行。

代码：https://github.com/rcore-os/rCore-Tutorial-v3/blob/ch3/os/src/task/switch.S

完整流程：

> 问题：为什么对比第二章的trap.S文件少了 mv sp, a0
>
> __restore在这里被两种情况复用了：
>
> 1. 正常从__alltraps走下来的trap_handler流程。如果是这种情况，trap_handler会在a0里返回之前通过mv a0, sp传进去的&mut TrapContext，所以这里sp和a0相同没有必要再mv sp, a0重新设置一遍。
>
>2. app第一次被__switch的时候通过__restore开始运行。这时候a0是个无关的数据（指向上一个>TaskContext的指针），这里再mv sp a0就不对了，而__restore要的TrapContext已经在>__switch的恢复过程中被放在sp上了。（这个sp就是初始化时写完TrapContext后的内核栈顶

