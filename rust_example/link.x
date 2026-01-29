/* link.x - Linker script additions for riscv-rt compatibility */
/* This is included AFTER memory.x which defines MEMORY regions */

/* Provide default handlers that can be overridden */
PROVIDE(_stext = ORIGIN(FLASH));
PROVIDE(_stack_start = ORIGIN(RAM) + LENGTH(RAM));
PROVIDE(_max_hart_id = 0);
PROVIDE(_hart_stack_size = 0x4000);  /* 16KB stack per hart */
PROVIDE(_heap_size = 0x10000);        /* 64KB heap */

/* Memory aliases for riscv-rt */
REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

/* AE350-specific: bootloader section must be first */
SECTIONS
{
    .bootloader ORIGIN(FLASH) :
    {
        KEEP(*(.bootloader))
        KEEP(*(.bootloader.*))
    } > FLASH

    /* Rest handled by riscv-rt's link.x */
}
INSERT BEFORE .text;
