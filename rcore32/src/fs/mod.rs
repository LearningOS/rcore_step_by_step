mod device;
mod structs;

use device::{ Device, MemBuf };
use structs::*;
use spin::RwLock;
use alloc::sync::Arc;

pub struct SimpleFileSystem {
    super_block : RwLock<SuperBlock>,
}

impl SimpleFileSystem {
    pub fn open(device: Arc<Device>) -> Option<Arc<Self>> {
        let super_block = device.load_struct::<SuperBlock>(BLKN_SUPER).unwrap();
        if !super_block.check() {   // 检查超级块是否格式正确
            println!("super block check failed !");
        }

        Some(Arc::new(SimpleFileSystem {
            super_block: RwLock::new(super_block),
        }))
    }
}

pub fn init() {
    let device = {
        extern "C"{
            fn _user_img_start();
            fn _user_img_end();
        }
        // 将存储磁盘文件的内存范围初始化为虚拟磁盘Membuf
        Arc::new(unsafe { MemBuf::new(_user_img_start , _user_img_end ) })
    };

    let sfs = SimpleFileSystem::open(device).expect("failed to open SFS");
}
