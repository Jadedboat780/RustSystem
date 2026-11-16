use crate::sync::Lazy;
use x86_64::{
    VirtAddr,
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        paging::{Mapper, Page, Size4KiB, mapper::UnmapError},
        tss::TaskStateSegment,
    },
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const PAGE_SIZE: usize = 4096;
const IST_STACK_SIZE: usize = PAGE_SIZE * 5;

#[repr(C, align(4096))]
struct IstStack {
    guard: [u8; PAGE_SIZE],
    stack: [u8; IST_STACK_SIZE],
}

static mut DOUBLE_FAULT_STACK: IstStack = IstStack {
    guard: [0; PAGE_SIZE],
    stack: [0; IST_STACK_SIZE],
};

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    let stack_start = VirtAddr::from_ptr(unsafe { &raw const DOUBLE_FAULT_STACK.stack });
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_start + IST_STACK_SIZE as u64;
    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
    (
        gdt,
        Selectors {
            code_selector,
            data_selector,
            tss_selector,
        },
    )
});

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::{
        segmentation::{CS, DS, ES, SS, Segment},
        tables::load_tss,
    };

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub fn double_fault_guard_page() -> Page {
    Page::containing_address(VirtAddr::from_ptr(unsafe { &raw const DOUBLE_FAULT_STACK.guard }))
}

pub fn protect_stacks(mapper: &mut impl Mapper<Size4KiB>) -> Result<(), UnmapError> {
    let (_, flush) = mapper.unmap(double_fault_guard_page())?;
    flush.flush();
    Ok(())
}
