use std::{sync::{Arc, RwLock}, thread::JoinHandle, time::Instant};
use slab::Slab;

use crate::{sharky_memory::*, sharky_data_types::*, sharky_vm::*, sharky_instruction_set::*};

const GC_COLLECTION_COUNT: usize = 1;

struct SharkyInterpreterThread {
    pub interpreter: SharkySyncedInterpreter,
    pub thread: JoinHandle<()>
}

impl SharkyInterpreterThread {
    pub fn new(synced: SharkySyncedApp, program: Arc<SharkyProgram>) -> Self {
        let interpreter = SharkyInterpreter::new_arc(program, synced.clone());

        let return_interpreter = Arc::clone(&interpreter);

        let thread = std::thread::spawn(move || {
            let inter = Arc::clone(&interpreter);
            loop {
                let mut handle = inter.write().unwrap();
                if !handle.is_running() { break; }
                handle.interpret();
            }
            inter.read().unwrap().print_debug();
        });

        SharkyInterpreterThread { 
            interpreter: return_interpreter, 
            thread: thread  
        }
    }

    pub fn complete(&self) -> bool {
        !self.interpreter.read().unwrap().is_running() &&
        self.thread.is_finished()
    }
}

#[derive(Default)]
pub struct SharkyApp {
    heap: SharkySyncedHeapStack,
    interpreters: Slab<SharkyInterpreterThread>,
    allocation_count: usize,
}

pub type SharkySyncedApp = Arc<RwLock<SharkyApp>>;

impl SharkyApp {
    pub fn new_arc() -> SharkySyncedApp {
        Arc::new(RwLock::new(Self::default()))
    }

    fn start_garbage_collector(app: SharkySyncedApp) { 
        /*
         * --- Garbage collection basic implementation ---
         * - Collect all dead/finished interpreters, remove them from the collector pool.
         * - Track the allocation count from all threads, if we exceed the threshold initialize a garbage collection.
         * - Collection is just searching each stack for a reference pointing to each heap. If a heap with no references
         * it's collected and we move on. 
        */
        let arc_self = app.clone();
        std::thread::spawn(move || {
            loop {
            { // block the write guard.
                let mut this = arc_self.write().unwrap();
                println!("Collecting interpreters"); 
                this.collect_dead_interpreters();
                if this.allocation_count >= GC_COLLECTION_COUNT {
                    println!("Deallocating!");
                    this.heap.read().unwrap().iter().for_each(|cell| {
                        if let Some(stack_lock) = cell {
                            let stack = stack_lock.read().unwrap();
                            stack.debug_print_stack();
                        }
                    });
                    this.allocation_count = 0; // restart the count 
                    let mut heap = this.heap.write().unwrap();
                    for i in 0..heap.len() {
                        if !this.check_for_heap_ref(i) {
                            let frame = &mut heap[i];
                            // NOTE: this SHOULD be safe, because we take a write lock on the vector so setting to 
                            // None should never effect users of the data given you would still need a read lock 
                            // on the vector preventing this operation. BUT any issues should be checked here.
                            *frame = None;
                        }
                    }
                }
            }
            
            std::thread::yield_now();
        }
        });
    }

    fn check_for_heap_ref(&self, address: SharkyHeapFrameIndex) -> bool {
        for (_, interpreter) in self.interpreters.iter() {
            if interpreter.interpreter.read().unwrap().has_reference(address) {
                return true;
            }
        }
        false
    }

    fn spawn_interpreter(synced: SharkySyncedApp, program: Arc<SharkyProgram>) {
        let mut this = synced.write().unwrap();
        this.interpreters.insert(SharkyInterpreterThread::new(synced.clone(), program));
    }

    fn await_processes(app_arc: SharkySyncedApp) { 
        loop {
            { // scope for lock
                let complete = {
                    let app = app_arc.read().unwrap();
                    app.interpreters.iter().all(|(_, interpreter)| {
                        interpreter.complete()
                    })
                };

                if complete { break; }
            }
            std::thread::yield_now();
        }

        let mut app = app_arc.write().unwrap();
        for thread in app.interpreters.drain() {
            thread.thread.join().unwrap();
        }
    }

    fn collect_dead_interpreters(&mut self) {
        let interpreters = &mut self.interpreters;
        let collection_list: Vec<usize> = interpreters
        .iter()
        .filter(|(_, interpreter)| { interpreter.complete() })
        .map(|(idx, _)| idx)
        .collect();
        for i in collection_list {
            interpreters.remove(i);
        }
    }

    pub fn init(program: Arc<SharkyProgram>) {
        let app = Self::new_arc();
        Self::start_garbage_collector(app.clone());
        Self::spawn_interpreter(app.clone(), program.clone());
        Self::await_processes(app);
    }

    pub fn new(&mut self) -> SharkyHeapFrameIndex {
        let mut heap = self.heap.write().unwrap();
        heap.push(Some(RwLock::new(SharkyStack::default())));
        self.allocation_count += 1;
        heap.len() - 1
    }
    
    pub fn get(&self, SharkyHeapAddress(v,  q): SharkyHeapAddress) -> Option<SharkyDataType> {
        let heap = self.heap.read().ok()?;
        let frame = 
        heap
        .get(v)?
        .as_ref()?.read().ok()?;
        frame.read(q).cloned()
    }

    pub fn set(&mut self, SharkyHeapAddress(v,  q): SharkyHeapAddress, val: &SharkyDataType) -> Option<()> {
        let heap = self.heap.read().ok()?;
        let mut frame =  
        heap
        .get(v)?
        .as_ref()?.write().ok()?;
        frame.set(q, val.clone());
        Some(())
    }
}
