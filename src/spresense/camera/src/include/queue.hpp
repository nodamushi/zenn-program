#ifndef CAMERA_QUEUE_HPP__
#define CAMERA_QUEUE_HPP__
#include <atomic>
#include <nuttx/semaphore.h>

namespace queue {

/**
 * 単一プロデューサー・単一コンシューマー（SPSC）向けのロックフリー風キュー。
 *
 * セマフォを使用して、データスロット数を制御します。
 *
 * - enqueue()：空きスロットがあるまで待機し、書き込みます。
 * - dequeue()：データがあるまで待機し、読み込みます。
 *
 * スレッド終了時には `ScopedNotify` を使ってセマフォを事後解放することで、
 * 対応スレッド側をブロックから解除可能です。

 * `ScopedNotify` は `producerBegin`, `consumerBegin` で作成できます。
 */
template <typename T, int N>
struct Queue {

  // スコープ終了時に対応スレッドを起こすためのRAII通知
  struct ScopedExit {
    ScopedExit(sem_t *s) noexcept : sem(s) {}
    ScopedExit(ScopedExit &&x) noexcept : sem(x.sem) { x.sem = nullptr; }
    ScopedExit &operator=(ScopedExit &&x) noexcept {
      if (&x == this)
        return *this;
      sem = x.sem;
      x.sem = nullptr;
      return *this;
    }
    ~ScopedExit() {
      if (sem)
        nxsem_post(sem);
    }

  private:
    ScopedExit(const ScopedExit &) = delete;
    ScopedExit &operator=(const ScopedExit &) = delete;
    sem_t *sem;
  };

  Queue() noexcept {
    nxsem_init(&rest, 0, N);
    nxsem_init(&count, 0, 0);
    head = tail = 0;
  }

  ~Queue() noexcept {
    nxsem_destroy(&rest);
    nxsem_destroy(&count);
  }

  void enqueue(const T &v) noexcept {
    nxsem_wait(&rest);
    buffer[head] = v;
    head = (head + 1) % N;
    nxsem_post(&count);
  }

  T dequeue() noexcept {
    nxsem_wait(&count);
    T v = buffer[tail];
    tail = (tail + 1) % N;
    nxsem_post(&rest);
    return v;
  }

  /** enqueue 側のRAIIオブジェクトを作成。2度以上呼ばないでください */
  ScopedExit producerBegin() {
    return ScopedExit(&count);
  }

  /** dequeue 側のRAIIオブジェクトを作成。2度以上呼ばないでください */
  ScopedExit consumerBegin() { return ScopedExit(&rest); }

private:
  T buffer[N];
  int head, tail;
  sem_t rest, count;
};

} // namespace queue

#endif