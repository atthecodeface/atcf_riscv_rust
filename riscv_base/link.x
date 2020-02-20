ENTRY(_start);
SECTIONS
{
  . = DEFINED(_pxebootStart) ? 0x4000:0x0;
  .start : {
    KEEP(*(.vectors));
    KEEP(*(.start));
    KEEP(*(.init));
    KEEP(*(.init.rust));
    . = ALIGN(4);
  }
  . = ALIGN(16);
  .text : {
    *(.text .text.*)
    . = ALIGN(4);
    *(.rodata*)
    *(.data*)
    . = ALIGN(4); /* required by lld */
   } 
  . = 0x3f00; /* Preserve 256 bytes at top of 16kB */
  . = DEFINED(_pxebootStart) ? 0x7f00:0x3f00;
  .stack : {
    _estack = .;
    _stack_base = .;
  }
  /* Discard .eh_frame, we are not doing unwind on panic so it is not needed */
  /DISCARD/ :
  {
    *(.eh_frame);
  }
/*
    *(.debug*);
    *(.symtab*);
    *(.comment*);
    *(.shstrtab*);

 . = 0x10000;
*/
}
