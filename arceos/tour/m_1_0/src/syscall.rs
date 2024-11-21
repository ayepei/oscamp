#![allow(dead_code)]

use axhal::arch::TrapFrame;
use axhal::trap::{register_trap_handler, SYSCALL,PAGE_FAULT};
use axerrno::LinuxError;
use axhal::paging::{MappingFlags,PageTable};
const SYS_EXIT: usize = 93;
use axhal::mem::VirtAddr;
use axtask::TaskExtRef;
#[register_trap_handler(SYSCALL)]
fn handle_syscall(tf: &TrapFrame, syscall_num: usize) -> isize {
    ax_println!("handle_syscall ...");
    let ret = match syscall_num {
        SYS_EXIT => {
            ax_println!("[SYS_EXIT]: process is exiting ..");
            axtask::exit(tf.arg0() as _)
        },
        _ => {
            ax_println!("Unimplemented syscall: {}", syscall_num);
            -LinuxError::ENOSYS.code() as _
        }
    };
    ret
}
#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(va:VirtAddr, access_flags: MappingFlags, is_user: bool) -> bool {
    let binding = axtask::current();
    let mut curr = binding.task_ext().aspace.lock();
    if is_user==true{
       return  curr.handle_page_fault(va,access_flags);
    }
    false
}