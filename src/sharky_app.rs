use std::{collections::HashMap, path::Path, sync::Arc, thread::JoinHandle, time::Instant};
use parking_lot::RwLock;
use slab::{Iter, Slab};

use crate::{sharky_data_types::*, sharky_memory::*, sharky_native::SharkyNativeLibrary, sharky_vm::*};

const GC_COLLECTION_COUNT: usize = 128;

struct SharkyTask {
    pub vm: SharkySyncedVM,
    pub thread: JoinHandle<()>
}

impl SharkyTask {
    pub fn new_subvm(vm: SharkySyncedVM) -> Self {
        let out_vm = Arc::clone(&vm);

        let thread = std::thread::spawn(move || {
            let personal_vm = Arc::clone(&vm);
            loop {
                let mut handle = personal_vm.write();
                if !handle.is_running() { break; }
                handle.interpret();
            }
            personal_vm.write().print_debug();
        });

        SharkyTask { 
            vm: out_vm, 
            thread: thread  
        }
    }

    pub fn new(heap: &SharkyHeap, program: Arc<SharkyProgram>, task_pool: &SharkyTaskPool) -> Self {
        let vm = SharkyVM::new_arc(program, heap, task_pool);
        Self::new_subvm(vm)
    }


    pub fn complete(&self) -> bool {
        self.thread.is_finished()
    }
}

#[derive(Default, Clone)]
pub struct SharkyTaskPool {
    tasks: SharkySynced<Slab<SharkyTask>>,
} 

impl SharkyTaskPool {
    pub fn new() -> Self {Self::default()}

    pub fn spawn_task(&mut self, heap: &SharkyHeap, program: Arc<SharkyProgram>) -> usize {
        self.tasks.write().insert(SharkyTask::new(heap, program, self))
    }

    pub fn spawn_subtask(&mut self, vm: SharkySyncedVM) -> usize {
        self.tasks.write().insert(SharkyTask::new_subvm(vm))
    }

    pub fn has_reference(&self, frame: SharkyHeapFrameIndex) -> bool {
        for (_, task) in self.tasks.read().iter() {
            if task.vm.read().has_reference(frame) {
                return true;
            }
        }
        false
    }

    pub fn complete(&self) -> bool {
        self.tasks.read().iter().all(|(_, interpreter)| { interpreter.complete() })
    }

    pub fn join(&mut self) {
        for thread in self.tasks.write().drain() {
            thread.thread.join().unwrap();
        }
    }

    pub fn collect_complete(&mut self) {
        let interpreters = &mut self.tasks.write();
        let collection_list: Vec<usize> = interpreters
        .iter()
        .filter(|(_, interpreter)| { interpreter.complete() })
        .map(|(idx, _)| idx)
        .collect();
        for i in collection_list {
            interpreters.remove(i);
        }
    }
}

pub struct SharkyFFI {
    natives: Arc<RwLock<HashMap<String, SharkyNativeLibrary>>>,
}

impl SharkyFFI {
    fn load_native(&mut self, path: &Path) {
        
    }
}

#[derive(Default)]
pub struct SharkyApp {
    heap: SharkyHeap,
    globals: Vec<SharkyHeapFrameIndex>,
    task_pool: SharkyTaskPool,
}


impl SharkyApp {
    pub fn new() -> Self {
        Self::default()
    }

    fn spawn_task(&mut self, program: Arc<SharkyProgram>) {
        self.task_pool.spawn_task(&self.heap, program);
    }

    fn garbage_collect(&mut self) {
        self.task_pool.collect_complete();
        if self.heap.get_allocation_count() < GC_COLLECTION_COUNT {
            return;
        }
        self.heap.reset_allocation_count();

        let mut to_collect: Vec<SharkyHeapFrameIndex> = vec![];

        for frame in self.heap.collect_heap_indexes() {
            if !self.heap.has_reference(frame) 
            && !self.task_pool.has_reference(frame) 
            && !self.globals.iter().any(move |v| {*v == frame}) {
                to_collect.push(frame) 
            }
        }

        for frame in to_collect {
            self.heap.free(frame);
        }
    }

    fn await_processes(&mut self) { 
        loop {
            if self.task_pool.complete() { break; }
            self.garbage_collect();
        }
        self.heap.print_debug();
        self.task_pool.join();
    }

    pub fn init(program: Arc<SharkyProgram>, mut global_frames: Vec<SharkyDataStack>) {
        let mut app = Self::new();

        for frame in global_frames.drain(..) {
            app.globals.push(app.heap.take_frame(frame));
        }

        app.spawn_task(program.clone());
        app.await_processes();
    }

}
