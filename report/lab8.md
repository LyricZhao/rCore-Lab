## rCore-Lab-8 文件系统

> 赵成钢 计75班 2017011362

#### 要求一：阅读文档并确保之前实验中实现了`sys_fork`
该作业分支是从`lab-5`的分支 check out 出来的，保证了`sys_fork`的实现。

#### 要求二：实现管道

- 对于管道的传输，我并没有用对这种文件保存 `offset` 之类的属性，而是直接用了 `VecDeque` ，并用了一层 `Arc<Mutex<VecDeque>>` 封装，在文件里面直接进行引用即可，需要注意的是文件是两个 `File`，但是引用的是一个双端队列。
- 这两个 File 也会共享一个 `Arc<Condvar>` 和一个 `Arc<Mutex<VecDeque>>`，用于堵塞读取和数据分享。
- 要写入的一端，直接拿到队列的锁之后对双端队列进行`push_back`，然后 `condvar.notify()`
- 要读取的一端，拿到锁之后 `pop_front`，然后如果没有就 `condvar.wait()`

- 思考题
  - 父进程还没写，子进程直接读怎么办？
    - 利用 `CondVar`，让线程放弃 CPU 资源；
    - 当有人写了之后，会进行 `notify`。
  - 如何保证不会发生 race condition ？
    - 中断的过程不会再被打断了，所以双端一定不会出现竞争的情况，之所以加锁是为了让可以在不同线程取可变引用，实际上不会发生竞争。
  - 在实现中是否曾遇到死锁？如果是，你是如何解决它的？
    - 遇到过，第一个情况是：
    ```rust
        if let Some(ch) = self.pipe.lock().pop_front() {
            return ch;
        } else {
            self.condvar.wait();
        }      
    ```
    注意到这里会发生死锁，因为 `if let` 中 `.lock` 在 `else` 结束之后才会放锁（生命周期），这个时候 `wait` 会进入父线程写的逻辑，父线程会拿不到锁。
    - 第二个情况是（我猜的可能的实现），如果只用一个 `File` 描述符，会在 `sys_read` 和 `sys_write` 的 `file.lock()` 的逻辑处发生死锁。所以这里我采用了两个 `File`，只需要注意可能死锁的情况（第一个情况）只有 `pipe: Arc<Mutex<VecDeque<u8>>>`，而 `Condvar` 的控制逻辑本身不在这部分的实现，正常使用就不会发生死锁。

