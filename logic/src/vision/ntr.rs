pub struct NTRPacket {
    seq: u32,
    typ: u32,
    cmd: u32,
    arg_priority: u32,
    arg_quality: u32,
    arg_qos: u32,
    args_extra: Vec<u32>,
    data_length: u32,
}

impl NTRPacket {
    pub const HDR_SIZE: usize = 4 + (4 * 3) + (4 * 16) + 4;

    pub fn heartbeat(seq: u32) -> Self {
        // TODO incrementing sequence
        NTRPacket {
            seq,
            typ: 0,
            cmd: 0,
            arg_priority: 0,
            arg_quality: 0,
            arg_qos: 0,
            args_extra: vec![],
            data_length: 0,
        }
    }

    pub fn init() -> Self {
        // Start Streaming Packet
        // seq = 3000
        // type = 0
        // cmd = 901
        // args[0] = top_screen<<8 | (priority%256)
        // args[1] = quality
        // args[2] = (qos*2)<<16
        let arg_priority = 0 << 8 | (2 % 256);
        let arg_quality = 20;
        let arg_qos = (16 * 2) << 16;
        NTRPacket {
            seq: 3000,
            typ: 0,
            cmd: 901,
            arg_priority,
            arg_quality,
            arg_qos,
            args_extra: vec![],
            data_length: 0,
        }
    }

    pub fn extra_len(&self) -> usize {
        self.data_length.try_into().unwrap()
    }

    pub fn from_wire(bytes: &[u8; Self::HDR_SIZE]) -> Option<Self> {
        let magic = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let seq = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let typ = u32::from_be_bytes(bytes[8..12].try_into().unwrap());
        let cmd = u32::from_be_bytes(bytes[12..16].try_into().unwrap());
        let mut args: Vec<_> = Vec::with_capacity(16);
        for i in 0..16 {
            let start = 16 + (i * 4);
            let end = start + 4;
            let arg = u32::from_be_bytes(bytes[start..end].try_into().unwrap());
            args.push(arg);
        }
        let extra_length = u32::from_le_bytes(bytes[80..84].try_into().unwrap());

        if magic == 0x78563412 {
            Some(Self {
                seq,
                typ,
                cmd,
                arg_priority: 0,
                arg_quality: 0,
                arg_qos: 0,
                args_extra: args,
                data_length: extra_length,
            })
        } else {
            None
        }
    }

    pub fn to_wire(&self) -> Vec<u8> {
        let mut buf = vec![];

        // Magic number
        buf.push(0x78);
        buf.push(0x56);
        buf.push(0x34);
        buf.push(0x12);

        // Sequence
        buf.extend_from_slice(&self.seq.to_le_bytes());
        // Type
        buf.extend_from_slice(&self.typ.to_le_bytes());
        // Command
        buf.extend_from_slice(&self.cmd.to_le_bytes());
        // Arg[0]
        buf.extend_from_slice(&self.arg_priority.to_le_bytes());
        // Arg[1]
        buf.extend_from_slice(&self.arg_quality.to_le_bytes());
        // Arg[2]
        buf.extend_from_slice(&self.arg_qos.to_le_bytes());
        // Args 3..16 and rest of packet
        while buf.len() < 84 {
            buf.push(0x00);
        }

        if self.args_extra.len() > 0 {
            log::error!("Extra arguments unsupported when converting to wire format");
        }
        if self.data_length != 0 {
            log::error!("Extra data unsupported when converting to wire format");
        }

        buf
    }
}
