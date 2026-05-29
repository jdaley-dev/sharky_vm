use std::{sync::{Arc, RwLock}, thread::JoinHandle, time::Instant};
use crate::{sharky_memory::*, sharky_data_types::*, sharky_vm::*, sharky_instruction_set::*};

const GC_COLLECTION_COUNT: usize = 2048;

#[derive(Default)]
pub struct SharkyApp {
    heap: SharkySyncedHeapStack,
    interpreters: Vec<Option<SharkySyncedInterpreter>>,
    threads: Vec<JoinHandle<()>>,
    allocation_count: usize,
}

pub type SharkySyncedApp = Arc<RwLock<SharkyApp>>;

// TODO: honestly its a bit late to refactor im tired, but the app really should just initialize with the garbage collector, a starting program, and the gc thread.
// no need for this mess. clean it up jay.

impl SharkyApp {
    pub fn new_arc() -> SharkySyncedApp {
        Arc::new(RwLock::new(Self::default()))
    }

    pub fn start_garbage_collector(app: SharkySyncedApp) {
        let arc_self = app.clone();
        std::thread::spawn(move || {
            loop {
            let mut this = arc_self.write().unwrap();
            this.collect_dead_interpreters();
            if this.allocation_count >= GC_COLLECTION_COUNT {
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
            std::thread::yield_now();
        }
        });
    }

    pub fn check_for_heap_ref(&self, address: SharkyHeapFrameIndex) -> bool {
        for interpreter_opt in self.interpreters.iter() {
            match interpreter_opt.as_ref() {
                Some(x) => {
                    if x.read().unwrap().has_reference(address) {
                        return true;
                    }
                },
                None => {}
            }
        }
        false
    }

    pub fn spawn_interpreter(&mut self, program: Arc<SharkyProgram>) {
        let interpreter = SharkyInterpreter::new_arc(program);
        self.interpreters.push(Some(Arc::clone(&interpreter)));
        self.threads.push(std::thread::spawn(move || {
            let inter = Arc::clone(&interpreter);
            let mut running = inter.read().unwrap().is_running();
            let start = Instant::now();
            while running {
                let mut handle = inter.write().unwrap();
                handle.interpret();
                running = handle.is_running();         
                if !running {
                    handle.print_debug();
                }       
            }
            let elapsed = start.elapsed().as_nanos();
            println!("Took: {}", elapsed)
        }));
    }

    pub fn await_processes(app_arc: SharkySyncedApp) { 
        loop {
            let complete = {
                let app = app_arc.read().unwrap();
                app.threads.iter().all(|t| t.is_finished())
            };

            if complete { break; }
            std::thread::yield_now();
        }

        let mut app = app_arc.write().unwrap();
        for thread in app.threads.drain(..) {
            thread.join().unwrap();
        }
    }

    pub fn collect_dead_interpreters(&mut self) {
        for interpreter_option in self.interpreters.iter_mut() {
            let mut kill = false;
            match interpreter_option.as_ref() 
            { 
                Some(interpreter_rw) => {
                    let interpreter = interpreter_rw.write().unwrap();
                    if !interpreter.is_running() {
                        kill = true;
                    }
                }, 
                None => {}
            }
            if kill {
                *interpreter_option = None;
            }
        }
    }

    pub fn allocate_frame(&mut self) -> SharkyHeapFrameIndex {
        let mut heap = self.heap.write().unwrap();
        heap.push(Some(RwLock::new(SharkyStack::default())));
        self.allocation_count += 1;
        heap.len() - 1
    }
    
    pub fn get(&self, address: SharkyHeapAddress) -> Option<SharkyDataType> {
        let heap = self.heap.read().ok()?;
        let frame = 
        heap
        .get(address.frame)?
        .as_ref()?.read().ok()?;
        frame.read(address.index).cloned()
    }

    pub fn set(&mut self, address: SharkyHeapAddress, val: &SharkyDataType) -> Option<()> {
        let heap = self.heap.read().ok()?;
        let mut frame =  
        heap
        .get(address.frame)?
        .as_ref()?.write().ok()?;
        frame.set(address.index, val.clone());
        Some(())
    }
}
