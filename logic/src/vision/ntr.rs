pub struct NTRPacket {
    seq: u32,
    typ: u32,
    cmd: u32,
    arg_priority: u32,
    arg_quality: u32,
    arg_qos: u32,
}

impl NTRPacket {
    pub fn heartbeat(seq: u32) -> Self {
        // TODO incrementing sequence
        NTRPacket {
            seq,
            typ: 0,
            cmd: 0,
            arg_priority: 0,
            arg_quality: 0,
            arg_qos: 0,
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
        let arg_priority = 1 << 8 | (2 % 256);
        let arg_quality = 30;
        let arg_qos = (16 * 2) << 16;
        NTRPacket {
            seq: 3000,
            typ: 0,
            cmd: 901,
            arg_priority,
            arg_quality,
            arg_qos,
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

        buf
    }
}
