use core::{
    arch::asm,
    panic,
    pin::Pin,
    ptr::{self, addr_eq, addr_of, addr_of_mut},
};

use common::println;

const MAX_PROCESSES: usize = 3;

#[derive(Debug, PartialEq, Clone)]

pub(crate) enum ProcessState {
    Unused,
    Runnable,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Process {
    pub pid: isize,
    pub state: ProcessState,
    pub sp: usize,
    pub stack: [u32; 8192 / 4], // 8 KB stack,
}

impl Process {
    pub const fn new(pid: isize) -> Process {
        Process {
            pid,
            state: ProcessState::Unused,
            sp: 0,
            stack: [0; 8192 / 4],
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum CurrentProcess {
    Idle,
    Process(usize),
}

impl CurrentProcess {
    pub fn as_isize(&self) -> isize {
        match self {
            CurrentProcess::Idle => -1,
            CurrentProcess::Process(pid) => (*pid) as isize,
        }
    }
}
#[derive(Debug)]
pub struct ProcessManager {
    processe_table: [Process; MAX_PROCESSES],
    // ref to the current process
    pub current_process: CurrentProcess,
    idle_process: Process,
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            processe_table: [Process::new(0), Process::new(1), Process::new(2)],
            current_process: CurrentProcess::Idle,
            idle_process: Process::new(-1),
        }
    }

    pub fn create_idle(&mut self, entry: usize) {
        println!("Creating idle process with PID: {}", self.idle_process.pid);
        self.idle_process.state = ProcessState::Runnable;
        let stack_btm = addr_of_mut!(self.idle_process.stack) as *mut u32;
        unsafe {
            let sp = stack_btm.add(self.idle_process.stack.len());
            *sp.offset(-1) = 0; // s11
            *sp.offset(-2) = 0; // s10
            *sp.offset(-3) = 0; // s9
            *sp.offset(-4) = 0; // s8
            *sp.offset(-5) = 0; // s7
            *sp.offset(-6) = 0; // s6
            *sp.offset(-7) = 0; // s5
            *sp.offset(-8) = 0; // s4
            *sp.offset(-9) = 0; // s3
            *sp.offset(-10) = 0; // s2
            *sp.offset(-11) = 0; // s1
            *sp.offset(-12) = 0; // s0
            *sp.offset(-13) = entry as u32; // ra
            self.idle_process.sp = sp.offset(-13) as usize;
        }
        println!("Idle process created");
    }

    pub fn create(&mut self, entry: usize) {
        for proc in self.processe_table.iter_mut() {
            if proc.state == ProcessState::Unused {
                println!("Creating process with PID: {}", proc.pid);
                proc.state = ProcessState::Runnable;
                let stack_btm = addr_of_mut!(proc.stack) as *mut u32;
                unsafe {
                    let sp = stack_btm.add(proc.stack.len());
                    *sp.offset(-1) = 0; // s11
                    *sp.offset(-2) = 0; // s10
                    *sp.offset(-3) = 0; // s9
                    *sp.offset(-4) = 0; // s8
                    *sp.offset(-5) = 0; // s7
                    *sp.offset(-6) = 0; // s6
                    *sp.offset(-7) = 0; // s5
                    *sp.offset(-8) = 0; // s4
                    *sp.offset(-9) = 0; // s3
                    *sp.offset(-10) = 0; // s2
                    *sp.offset(-11) = 0; // s1
                    *sp.offset(-12) = 0; // s0
                    *sp.offset(-13) = entry as u32; // ra
                    proc.sp = sp.offset(-13) as usize;
                }
                println!("Process created");
                return;
            }
        }
        panic!("No more processes can be created");
    }

    pub fn schedule(&mut self) {
        let prev = self.current_process.clone();
        let mut next = CurrentProcess::Idle;
        for i in 1..MAX_PROCESSES {
            let next_pid = (self.current_process.as_isize() + i as isize) as usize % MAX_PROCESSES;
            if self.processe_table[next_pid].state == ProcessState::Runnable {
                next = CurrentProcess::Process(next_pid);
                break;
            }
        }

        // update sscratch register

        if next == CurrentProcess::Idle {
            unsafe {
                asm!("csrw sscratch, {0}", in(reg) addr_of!(self.idle_process.stack[0]).add(self.idle_process.stack.len()));
            }
        } else {
            unsafe {
                asm!("csrw sscratch, {0}", in(reg) addr_of!(self.processe_table[next.as_isize() as usize].stack[0]).add(self.processe_table[next.as_isize() as usize].stack.len()));
            }
        }

        match (prev.clone(), next.clone()) {
            (CurrentProcess::Idle, CurrentProcess::Process(next_pid)) => {
                //dbg println!("Switching from idle to process {}", next_pid);
                self.current_process = next;
                unsafe {
                    switch_context_arch(
                        addr_of_mut!(self.idle_process.sp),
                        addr_of_mut!(self.processe_table[next_pid].sp),
                    );
                }
            }
            (CurrentProcess::Process(prev_pid), CurrentProcess::Idle) => {
                //dbg println!("Switching from process {} to idle", prev_pid);
                self.current_process = next;
                unsafe {
                    switch_context_arch(
                        addr_of_mut!(self.processe_table[prev_pid].sp),
                        addr_of_mut!(self.idle_process.sp),
                    );
                }
            }
            (CurrentProcess::Process(prev_pid), CurrentProcess::Process(next_pid))
                if prev_pid != next_pid =>
            {
                //dbg println!("Switching from process {} to process {}",prev_pid, next_pid);
                self.current_process = next;
                unsafe {
                    switch_context_arch(
                        addr_of_mut!(self.processe_table[prev_pid].sp),
                        addr_of_mut!(self.processe_table[next_pid].sp),
                    );
                }
            }
            (CurrentProcess::Process(pid), CurrentProcess::Process(next_pid))
                if pid == next_pid =>
            {
                //dbg println!("Continuing with process {}", pid);
            }
            _ => panic!("Invalid state"),
        }
    }

    pub fn get_current(&self) -> &Process {
        match self.current_process {
            CurrentProcess::Idle => &self.idle_process,
            CurrentProcess::Process(pid) => &self.processe_table[pid],
        }
    }
}

#[naked]
#[no_mangle]
extern "C" fn switch_context_arch(_prev_sp: *mut usize, next_sp: *const usize) {
    unsafe {
        asm!(
            // Save the current context
            "addi sp, sp, -13 * 4",
            "sw ra, 0 * 4(sp)",
            "sw s0, 1 * 4(sp)",
            "sw s1, 2 * 4(sp)",
            "sw s2, 3 * 4(sp)",
            "sw s3, 4 * 4(sp)",
            "sw s4, 5 * 4(sp)",
            "sw s5, 6 * 4(sp)",
            "sw s6, 7 * 4(sp)",
            "sw s7, 8 * 4(sp)",
            "sw s8, 9 * 4(sp)",
            "sw s9, 10 * 4(sp)",
            "sw s10, 11 * 4(sp)",
            "sw s11, 12 * 4(sp)",
            "sw sp, (a0)",
            // Load the next context
            "lw sp, (a1)",
            "lw ra, 0 * 4(sp)",
            "lw s0, 1 * 4(sp)",
            "lw s1, 2 * 4(sp)",
            "lw s2, 3 * 4(sp)",
            "lw s3, 4 * 4(sp)",
            "lw s4, 5 * 4(sp)",
            "lw s5, 6 * 4(sp)",
            "lw s6, 7 * 4(sp)",
            "lw s7, 8 * 4(sp)",
            "lw s8, 9 * 4(sp)",
            "lw s9, 10 * 4(sp)",
            "lw s10, 11 * 4(sp)",
            "lw s11, 12 * 4(sp)",
            "addi sp, sp, 13 * 4",
            "ret",
            options(noreturn),
        );
    }
}
