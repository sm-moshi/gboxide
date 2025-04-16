/// Memory interface abstraction between CPU and memory/IO
pub trait MemoryBus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}
