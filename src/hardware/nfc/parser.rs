#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum PayloadType {
    Text = 0x54,
    URL = 0x55,
    Unknown = 0xff,
}

impl From<i32> for PayloadType {
    fn from(v: i32) -> Self {
        match v {
            x if x == PayloadType::Text as i32 => PayloadType::Text,
            x if x == PayloadType::URL as i32 => PayloadType::URL,
            _ => PayloadType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct PayloadValue {
    pub raw_data: Vec<u8>,
    pub payload_type: PayloadType,
    pub data: String,
}

#[derive(Debug)]
pub struct ParseResult {
    pub message_type: u8,
    pub message_length: usize,
    pub records: Vec<PayloadValue>,
}

#[derive(Debug)]
pub enum NDEFParseState {
    MessageHeader,
    MessageType,
    MessageID,
    MessagePayload,
}

struct NextMessageMeta {
    block: MessageHeader,
    type_len: i32,
    payload_len: i32,
    id_len: i32,
}

fn read_message_header(data: &[u8], data_index: &mut usize) -> NextMessageMeta {
    let block = parse_ndef_message_block(data[*data_index]);
    *data_index += 1;

    let type_len = data[*data_index].into();
    *data_index += 1;

    let payload_len = if block.short_record() {
        let len = data[*data_index].into();
        *data_index += 1;
        len
    } else {
        let len = ((i32::from(data[*data_index])) << 24)
            + (i32::from(data[*data_index + 1]) << 16)
            + (i32::from(data[*data_index + 2]) << 8)
            + i32::from(data[*data_index + 3]);
        *data_index += 4;
        len
    };

    let id_len = if block.has_id_length() {
        let len = data[*data_index].into();
        *data_index += 1;
        len
    } else {
        -1
    };

    NextMessageMeta {
        block,
        type_len,
        payload_len,
        id_len,
    }
}

fn read_message_type(data: &[u8], data_index: &mut usize, type_len: i32) -> i32 {
    let mut message_type = 0;
    for n in (0..type_len).rev() {
        message_type += i32::from(data[*data_index]) << (n * 8);
        *data_index += 1;
    }
    message_type
}

fn skip_message_id(data_index: &mut usize, id_len: i32) {
    for _ in (0..id_len).rev() {
        *data_index += 1;
    }
}

#[allow(clippy::cast_sign_loss)]
fn read_payload_bytes(data: &[u8], data_index: &mut usize, payload_len: i32) -> Vec<u8> {
    let mut raw = Vec::with_capacity(payload_len.max(0) as usize);
    for _ in 0..payload_len {
        raw.push(data[*data_index]);
        *data_index += 1;
    }
    raw
}

fn decode_payload(payload_type: PayloadType, raw_data: &[u8]) -> String {
    match payload_type {
        PayloadType::Text => {
            let mut data = String::new();
            let encoding_length = raw_data[0];
            for &byte in raw_data.iter().skip(encoding_length as usize + 1) {
                data.push(byte as char);
            }
            data
        }
        PayloadType::URL => {
            let mut data = String::from(get_uri_protocol(raw_data[0]));
            for &byte in raw_data.iter().skip(1) {
                data.push(byte as char);
            }
            data
        }
        PayloadType::Unknown => String::new(),
    }
}

fn read_ndef_header(data: &[u8]) -> (u8, usize, usize) {
    let message_type = data[0];
    if data[1] > 0xfe {
        let message_length = ((data[2] as usize) << 8) + (data[3] as usize);
        (message_type, message_length, 4)
    } else {
        (message_type, data[1].into(), 2)
    }
}

/// Parses NFC data into NDEF structure
#[must_use]
pub fn parse_nfc_data(data: &[u8]) -> ParseResult {
    let (message_type, message_length, data_offset) = read_ndef_header(data);
    let mut parse_result: ParseResult = ParseResult {
        message_type,
        message_length,
        records: vec![],
    };

    let mut parse_record_index: usize = 0;
    let mut data_index: usize = data_offset;
    let mut data_parse_state: NDEFParseState = NDEFParseState::MessageHeader;

    let mut next_message_message_block = MessageHeader(0);
    let mut next_message_type_length: i32 = -1;
    let mut next_message_type: i32 = -1;
    let mut next_message_id_length: i32 = -1;
    let mut next_message_payload_length: i32 = -1;

    while data_index < parse_result.message_length + data_offset {
        data_parse_state = match data_parse_state {
            NDEFParseState::MessageHeader => {
                let meta = read_message_header(data, &mut data_index);
                next_message_message_block = meta.block;
                next_message_type_length = meta.type_len;
                next_message_payload_length = meta.payload_len;
                next_message_id_length = meta.id_len;

                NDEFParseState::MessageType
            }
            NDEFParseState::MessageType => {
                next_message_type =
                    read_message_type(data, &mut data_index, next_message_type_length);

                if next_message_message_block.has_id_length() {
                    NDEFParseState::MessageID
                } else {
                    NDEFParseState::MessagePayload
                }
            }
            NDEFParseState::MessageID => {
                skip_message_id(&mut data_index, next_message_id_length);

                NDEFParseState::MessagePayload
            }
            NDEFParseState::MessagePayload => {
                if !next_message_message_block.chunked()
                    || parse_result.records.len() <= parse_record_index
                {
                    parse_result.records.push(PayloadValue {
                        raw_data: vec![],
                        payload_type: PayloadType::Unknown,
                        data: String::new(),
                    });
                }

                parse_result.records[parse_record_index].payload_type =
                    PayloadType::from(next_message_type);

                let raw_data =
                    read_payload_bytes(data, &mut data_index, next_message_payload_length);
                parse_result.records[parse_record_index].raw_data = raw_data;
                parse_result.records[parse_record_index].data = decode_payload(
                    parse_result.records[parse_record_index].payload_type,
                    &parse_result.records[parse_record_index].raw_data,
                );

                if next_message_message_block.message_end() && !next_message_message_block.chunked()
                {
                    break;
                }
                if !next_message_message_block.chunked() {
                    parse_record_index += 1;
                }

                NDEFParseState::MessageHeader
            }
        };
    }
    parse_result
}

#[derive(Debug, Clone, Copy)]
pub struct MessageHeader(u8);

impl MessageHeader {
    fn message_end(self) -> bool {
        (self.0 & 0b0100_0000) != 0
    }

    fn chunked(self) -> bool {
        (self.0 & 0b0010_0000) != 0
    }

    fn short_record(self) -> bool {
        (self.0 & 0b0001_0000) != 0
    }

    fn has_id_length(self) -> bool {
        (self.0 & 0b0000_1000) != 0
    }
}

#[must_use]
pub fn parse_ndef_message_block(byte: u8) -> MessageHeader {
    MessageHeader(byte)
}

#[must_use]
pub fn get_uri_protocol(identifier: u8) -> &'static str {
    match identifier {
        0x01 => "http://www.",
        0x02 => "https://www.",
        0x03 => "http://",
        0x04 => "https://",
        0x05 => "tel:",
        0x06 => "mailto:",
        0x07 => "ftp://anonymous:anonymous@",
        0x08 => "ftp://ftp.",
        0x09 => "ftps://",
        0x0A => "sftp://",
        0x0B => "smb://",
        0x0C => "nfs://",
        0x0D => "ftp://",
        0x0E => "dav://",
        0x0F => "news:",
        0x10 => "telnet://",
        0x11 => "imap:",
        0x12 => "rtsp://",
        0x13 => "urn:",
        0x14 => "pop:",
        0x15 => "sip:",
        0x16 => "sips:",
        0x17 => "tftp:",
        0x18 => "btspp://",
        0x19 => "btl2cap://",
        0x1A => "btgoep://",
        0x1B => "tcpobex://",
        0x1C => "irdaobex://",
        0x1D => "file://",
        0x1E => "urn: epc: id:",
        0x1F => "urn: epc: tag:",
        0x20 => "urn: epc: pat:",
        0x21 => "urn: epc: raw:",
        0x22 => "urn: epc:",
        0x23 => "urn: nfc:",
        _ => "",
    }
}
