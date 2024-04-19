use core::ptr::NonNull;

use alloc::{boxed::Box, vec::Vec};
use driver_common::{BaseDriverOps, DevError};
use e1000_driver::e1000::E1000Device;
pub use e1000_driver::e1000::KernelFunc;

use crate::{EthernetAddress, NetBufPtr, NetDriverOps};

extern crate alloc;

pub struct E1000Nic<'a, K: KernelFunc> {
    inner: E1000Device<'a, K>,
}

impl<'a, K: KernelFunc> E1000Nic<'a, K> {
    pub fn init(mut kfn: K, mapped_regs: usize) -> driver_common::DevResult<Self> {
        Ok(Self {
            inner: E1000Device::<K>::new(kfn, mapped_regs).map_err(|err| {
                log::error!("Failed to initialize e1000 device: {:?}", err);
                DevError::BadState
            })?,
            // rx_buffer_queue: VecDeque::with_capacity(RX_BUFFER_SIZE),
        })
    }
}



unsafe impl<'a, K: KernelFunc> Send for E1000Nic<'a, K> {}
unsafe impl<'a, K: KernelFunc> Sync for E1000Nic<'a, K> {}

impl<'a, K: KernelFunc> BaseDriverOps for E1000Nic<'a, K> {
    fn device_name(&self) -> &str {
        "Intel E1000 Net Driver"
    }
    
    fn device_type(&self) -> driver_common::DeviceType {
        driver_common::DeviceType::Net
    }
}

impl<'a, K: KernelFunc> NetDriverOps for E1000Nic<'a, K> {
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress([0x52, 0x54, 0x00, 0x6c, 0xf8, 0x88])
    }

    fn can_transmit(&self) -> bool {
        true
    }

    fn can_receive(&self) -> bool {
        true
    }

    fn rx_queue_size(&self) -> usize {
        256
    }

    fn tx_queue_size(&self) -> usize {
        256
    }

    fn recycle_rx_buffer(&mut self, rx_buf: NetBufPtr) -> driver_common::DevResult {
        drop(rx_buf);
        Ok(())
    }

    fn recycle_tx_buffers(&mut self) -> driver_common::DevResult {
        Ok(())
    }

    fn transmit(&mut self, tx_buf: NetBufPtr) -> driver_common::DevResult {
        self.inner.e1000_transmit(tx_buf.packet());
        Ok(())
    }

    fn receive(&mut self) -> driver_common::DevResult<NetBufPtr> {
        match self.inner.e1000_recv() {
            Some(packets) => {
                let sum = packets.iter().map(|p| p.len()).sum();
                let mut buffer = Box::new(Vec::with_capacity(sum));
                let mut offset = 0;
                for packet in packets.iter() {
                    let len = packet.len();
                    buffer[offset..offset+len].copy_from_slice(packet);
                    offset += len;
                }

                Ok(NetBufPtr::new(NonNull::dangling(), NonNull::new(Box::into_raw(buffer) as *mut u8).unwrap(), sum))
            },
            None => Err(DevError::Again)
        }
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> driver_common::DevResult<NetBufPtr> {
        Err(DevError::NoMemory)
    }
}
