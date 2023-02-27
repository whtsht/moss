use crate::error::{ErrorKind, Result};

use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
use spin::{Lazy, Mutex};

static TASK_QUEUE: Lazy<Arc<ArrayQueue<TaskId>>> = Lazy::new(|| Arc::new(ArrayQueue::new(100)));

static TASKS: Lazy<Mutex<BTreeMap<TaskId, Task>>> = Lazy::new(|| Mutex::new(BTreeMap::new()));

static WAKER_CACHE: Lazy<Mutex<BTreeMap<TaskId, Waker>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub fn spawn(task: Task) -> Result<()> {
    TASK_QUEUE
        .push(task.id)
        .or(Err(ErrorKind::TaskQueueIsFull))?;

    if TASKS.lock().insert(task.id, task).is_some() {
        panic!("task with same ID already in tasks");
    }

    Ok(())
}

pub fn run_ready_tasks() {
    while let Some(task_id) = TASK_QUEUE.pop() {
        let mut task = match { TASKS.lock().remove(&task_id) } {
            Some(task) => task,
            None => continue,
        };

        let mut waker_cache = { WAKER_CACHE.lock() };

        let waker = waker_cache
            .entry(task_id)
            .or_insert_with(|| TaskWaker::new(task_id, TASK_QUEUE.clone()));

        let mut context = Context::from_waker(&waker);
        match task.poll(&mut context) {
            Poll::Ready(()) => {
                waker_cache.remove(&task_id);
            }
            Poll::Pending => {
                TASKS.lock().insert(task_id, task);
            }
        }
    }
}

fn sleep_if_idle() {
    use x86_64::instructions::interrupts::{self, enable_and_hlt};

    interrupts::disable();
    if TASK_QUEUE.is_empty() {
        enable_and_hlt();
    } else {
        interrupts::enable();
    }
}

pub fn run() -> ! {
    loop {
        run_ready_tasks();
        sleep_if_idle();
    }
}

#[allow(dead_code)]
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task()
    }
}
