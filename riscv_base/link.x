ENTRY(_start);
SECTIONS
{
  . = 0x0;
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
    . = ALIGN(4); /* required by lld */
   } 
  . = 0x800;
  .stack : {
    _estack = .;
    _stack_base = .;
    *(.stack)
    PROVIDE(_stack_end$ = . + 0x200);
  }
  /* Discard .eh_frame, we are not doing unwind on panic so it is not needed */
  /DISCARD/ :
  {
    *(.eh_frame);
  }
}
