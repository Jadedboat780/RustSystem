use crate::{allocator, console::WRITER, interrupts, logger, print};
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping};
use kernel_lib::{
    gdt,
    paging::{self, BootInfoFrameAllocator},
    pit, serial, time,
};
use log::LevelFilter;
use x86_64::{VirtAddr, structures::paging::OffsetPageTable};

pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

pub struct Kernel {
    mapper: OffsetPageTable<'static>,
    frame_allocator: BootInfoFrameAllocator,
}

impl Kernel {
    pub fn init(boot_info: &'static mut BootInfo) -> Self {
        if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
            WRITER.lock().attach_framebuffer(framebuffer);
        }

        logger::init(LevelFilter::Info);
        gdt::init();
        pit::init();
        time::init();
        interrupts::init_idt();
        unsafe {
            let mut pics = interrupts::PICS.lock();
            pics.initialize();
            pics.write_masks(0b1110_1100, 0b1111_1111);
        }
        serial::enable_receive_interrupt();

        let phys_mem_offset = boot_info
            .physical_memory_offset
            .into_option()
            .expect("Bootloader did not map physical memory");
        let mut mapper = unsafe { paging::init(VirtAddr::new(phys_mem_offset)) };
        let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_regions);
        if let Err(error) = gdt::protect_stacks(&mut mapper) {
            log::warn!("Failed to unmap the double fault stack guard page: {error:?}");
        }
        allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

        x86_64::instructions::interrupts::enable();

        Self { mapper, frame_allocator }
    }

    pub fn mapper(&mut self) -> &mut OffsetPageTable<'static> {
        &mut self.mapper
    }

    pub fn frame_allocator(&mut self) -> &mut BootInfoFrameAllocator {
        &mut self.frame_allocator
    }

    pub fn run(self) -> ! {
        print!("{}$ ", time::now());
        hlt_loop();
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
