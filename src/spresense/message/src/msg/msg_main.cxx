
#include <atomic>
#include <nuttx/config.h>
#include <nuttx/semaphore.h>
#include <pthread.h>
#include <stdio.h>
#include <sys/time.h>
#include <sched.h>
#include <sys/types.h>

/**
 * nxsem で作成したメッセージキュー
 */
struct NxsemMsg {
  using Self = NxsemMsg;

  NxsemMsg(unsigned int size) noexcept {
    nxsem_init(&used_, 0, 0);
    nxsem_init(&rest_, 0, size);
  }

  ~NxsemMsg() {
    nxsem_destroy(&used_);
    nxsem_destroy(&rest_);
  }

  struct Sender {
    Sender(Self &parent) : used_(&parent.used_), rest_(&parent.rest_) {}
    ~Sender() { nxsem_post(used_); } // dead lock 回避

    // ※本来ならなにかデータも送る
    void push() noexcept {
      nxsem_wait(rest_);
      nxsem_post(used_);
    }

  private:
    sem_t *used_;
    sem_t *rest_;
  };

  struct Receiver {
    Receiver(Self &parent) : used_(&parent.used_), rest_(&parent.rest_) {}
    ~Receiver() { nxsem_post(rest_); } // dead lock 回避

    // ※本来ならなにかデータも受信する
    void pop() noexcept {
      nxsem_wait(used_);
      nxsem_post(rest_);
    }

  private:
    sem_t *used_;
    sem_t *rest_;
  };

private:
  // 面倒なので copy, move 禁止
  NxsemMsg(const Self &) = delete;
  NxsemMsg(Self &&) = delete;
  sem_t used_;
  sem_t rest_;
};


/**
 * std::atomic で実装した全く同じ内容。
 * Sender: 1スレッド, Receiver: 1スレッドに限定される
 */
struct AtomicMsg {
  using Self = AtomicMsg;

  AtomicMsg(unsigned int size) noexcept : used_(0), rest_(size) {}

  struct Sender {
    Sender(Self &parent) : used_(parent.used_), rest_(parent.rest_) {}
    ~Sender() { used_.fetch_add(1); } // デッドロック回避

    // 本来ならなにかデータも送る
    void push() noexcept {
      while (1) {
        uint32_t v = rest_.load();
        if (v != 0) {
          //※ Sender は1スレッド想定
          //※ push するのが複数いる場合は compare_exchange_weak を使うこと
          rest_.fetch_sub(1);
          break;
        }
      }
      used_.fetch_add(1);
    }

  private:
    std::atomic_uint32_t &used_;
    std::atomic_uint32_t &rest_;
  };

  struct Receiver {
    Receiver(Self &parent) : used_(parent.used_), rest_(parent.rest_) {}
    ~Receiver() { rest_.fetch_add(1); }// デッドロック回避

    // 本来ならなにかデータも送る
    void pop() noexcept {
      while (1) {
        uint32_t v = used_.load();
        if (v != 0) {
          //※ Receiver は1スレッド想定
          //※ push するのが複数いる場合は compare_exchange_weak を使うこと
          used_.fetch_sub(1);
          break;
        }
      }
      rest_.fetch_add(1);
    }

  private:
    std::atomic_uint32_t &used_;
    std::atomic_uint32_t &rest_;
  };

private:
  std::atomic_uint32_t used_;
  std::atomic_uint32_t rest_;
};

/**
 * 送受信スレッドに渡す引数
 */
struct Arg {
  Arg(unsigned int size, uint32_t l)
      : loop(l), n(size), a(size), recv_time_n(0), recv_time_a(0),
        send_time_n(0), send_time_a(0) {}

  uint32_t loop;
  NxsemMsg n;
  AtomicMsg a;
  // ------- result---------------
  float recv_time_n;
  float recv_time_a;
  float send_time_n;
  float send_time_a;
};

// 受信スレッド
void *receiverThread(void *a) {
  Arg &arg = *(Arg *)a;
  // -------------------------------------
  struct timeval start_a;
  {
    AtomicMsg::Receiver recv(arg.a);
    for (uint32_t i = 0; i < arg.loop; i++) {
      recv.pop();
      if (i == 0)
        gettimeofday(&start_a, NULL);
    }
  }
  struct timeval end_a;
  gettimeofday(&end_a, NULL);

  // -------------------------------------
  struct timeval start_n;
  {
    NxsemMsg::Receiver recv(arg.n);
    for (uint32_t i = 0; i < arg.loop; i++) {
      recv.pop();
      if (i == 0)
        gettimeofday(&start_n, NULL);
    }
  }
  struct timeval end_n;
  gettimeofday(&end_n, NULL);

  // -------------------------------------
  arg.recv_time_n = (end_n.tv_sec - start_n.tv_sec) +
                    (end_n.tv_usec - start_n.tv_usec) / 1000000.0f;
  arg.recv_time_a = (end_a.tv_sec - start_a.tv_sec) +
                    (end_a.tv_usec - start_a.tv_usec) / 1000000.0f;
  return nullptr;
}

// 送信スレッド
void *senderThread(void *a) {
  Arg &arg = *(Arg *)a;

  // -------------------------------------
  struct timeval start_a;
  {
    AtomicMsg::Sender send(arg.a);
    for (uint32_t i = 0; i < arg.loop; i++) {
      send.push();
      if (i == 0)
        gettimeofday(&start_a, NULL);
    }
  }
  struct timeval end_a;
  gettimeofday(&end_a, NULL);

  // -------------------------------------
  struct timeval start_n;
  {
    NxsemMsg::Sender send(arg.n);
    for (uint32_t i = 0; i < arg.loop; i++) {
      send.push();
      if (i == 0)
        gettimeofday(&start_n, NULL);
    }
  }
  struct timeval end_n;
  gettimeofday(&end_n, NULL);


  // -------------------------------------
  arg.send_time_n = (end_n.tv_sec - start_n.tv_sec) +
                    (end_n.tv_usec - start_n.tv_usec) / 1000000.0f;
  arg.send_time_a = (end_a.tv_sec - start_a.tv_sec) +
                    (end_a.tv_usec - start_a.tv_usec) / 1000000.0f;
  return nullptr;
}


// main関数
extern "C" int main(int argc, FAR char *argv[]) {
  Arg arg(4, 100000);

  struct sched_param sparam = {};
  sparam.sched_priority = 110;
  pthread_attr_t attr;
  pthread_attr_init(&attr);
  pthread_attr_setschedparam(&attr, &sparam);

  pthread_t recv = -1;
  cpu_set_t cpuset;
  CPU_ZERO(&cpuset);
  CPU_SET(3, &cpuset);
  pthread_attr_setaffinity_np(&attr, sizeof(cpuset), &cpuset);

  pthread_create(&recv, &attr, receiverThread, (void *)&arg);

  pthread_t send = -1;
  CPU_ZERO(&cpuset);
  CPU_SET(4, &cpuset);
  pthread_attr_setaffinity_np(&attr, sizeof(cpuset), &cpuset);
  pthread_create(&send, &attr, senderThread, (void *)&arg);

  pthread_join(recv, nullptr);
  pthread_join(send, nullptr);
  printf("Recv: Nxsem  %f[s]\n", arg.recv_time_n);
  printf("Send: Nxsem  %f[s]\n", arg.send_time_n);
  printf("\n");
  printf("Recv: Atomic %f[s]\n", arg.recv_time_a);
  printf("Send: Atomic %f[s]\n", arg.send_time_a);

  return 0;
}
