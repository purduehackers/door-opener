#[derive(Debug)]
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
    pub data: String
}

#[derive(Debug)]
pub struct ParseResult {
    pub message_type: u8,
    pub message_length: usize,
    pub records: Vec<PayloadValue>
}

#[derive(Debug)]
pub enum NDEFParseState {
    MessageHeader,
    MessageType,
    MessageID,
    MessagePayload,
}

pub fn parse_nfc_data(data: Vec<u8>) -> Result<ParseResult, std::io::Error> {
    //Pull TLV Metadata

    let mut parse_record_index: usize = 0;

    let mut parse_result: ParseResult = ParseResult {
        message_type: data[0],
        message_length: 0,
        records: vec![]
    };

    let data_offset: usize = if data[1] > 0xfe {
        parse_result.message_length = ((data[2] as usize) << 8) + (data[3] as usize);

        4
    } else {
        parse_result.message_length = data[1].into();

        2
    };

    //Start NDEF State Machine

    let mut data_index: usize = data_offset;
    let mut data_parse_state: NDEFParseState = NDEFParseState::MessageHeader;

    let mut next_message_message_block: MessageBlock = MessageBlock {
        //message_begin: false,
        message_end: false,
        chunk_flag: false,
        short_record: false,
        id_length: false,
        //type_name_format: 0
    };
    let mut next_message_type_length: i32 = -1;
    let mut next_message_type: i32 = -1;
    let mut next_message_id_length: i32 = -1;
    //let mut next_message_id: i32 = -1;
    let mut next_message_payload_length: i32 = -1;

    while data_index < parse_result.message_length + data_offset {
        data_parse_state = match data_parse_state {
            NDEFParseState::MessageHeader => {
                next_message_message_block = parse_ndef_message_block(data[data_index]);
                data_index += 1;

                next_message_type_length = data[data_index].into();
                data_index += 1;

                if next_message_message_block.short_record {
                    next_message_payload_length = data[data_index].into();
                    data_index += 1;
                } else {
                    next_message_payload_length = 
                        ((data[data_index    ] as i32) << 24) + 
                        ((data[data_index + 1] as i32) << 16) + 
                        ((data[data_index + 2] as i32) << 8) + 
                         (data[data_index + 3] as i32);
                    data_index += 4;
                }

                next_message_id_length = -1;

                if next_message_message_block.id_length {
                    next_message_id_length = data[data_index].into();
                    data_index += 1;
                }

                NDEFParseState::MessageType
            },
            NDEFParseState::MessageType => {
                next_message_type = 0;

                for n in (0..next_message_type_length).rev() {
                    next_message_type += (data[data_index] as i32) << (n * 8);
                    data_index += 1;
                }

                if next_message_message_block.id_length {
                    NDEFParseState::MessageID
                } else {
                    NDEFParseState::MessagePayload
                }
            },
            NDEFParseState::MessageID => {
                //next_message_id = 0;

                for _ in (0..next_message_id_length).rev() {
                    //next_message_id += (data[data_index] as i32) << (n * 8);
                    data_index += 1;
                }

                NDEFParseState::MessagePayload
            },
            NDEFParseState::MessagePayload => {
                if !next_message_message_block.chunk_flag || parse_result.records.len() <= parse_record_index {
                    parse_result.records.push(PayloadValue {
                        raw_data: vec![],
                        payload_type: PayloadType::Unknown,
                        data: "".to_string()
                    });
                }

                parse_result.records[parse_record_index].payload_type = PayloadType::from(next_message_type);

                for _ in 0..next_message_payload_length {
                    parse_result.records[parse_record_index].raw_data.push(data[data_index]);
                    data_index += 1;
                }

                match parse_result.records[parse_record_index].payload_type {
                    PayloadType::Text => {
                        let encoding_length = parse_result.records[parse_record_index].raw_data[0];

                        for i in (encoding_length as usize + 1)..parse_result.records[parse_record_index].raw_data.len() {
                            let parsed_char =  parse_result.records[parse_record_index].raw_data[i] as char;
                            
                            parse_result.records[parse_record_index].data.push(parsed_char);
                        }
                    }, 
                    PayloadType::URL => {
                        parse_result.records[parse_record_index].data = String::from(
                            get_uri_protocol(parse_result.records[parse_record_index].raw_data[0])
                        );

                        for i in 1..parse_result.records[parse_record_index].raw_data.len() {
                            let parsed_char =  parse_result.records[parse_record_index].raw_data[i] as char;
                            
                            parse_result.records[parse_record_index].data.push(parsed_char);
                        }
                    }
                    _ => { }
                }

                if next_message_message_block.message_end && !next_message_message_block.chunk_flag {
                    break;
                } else {
                    if !next_message_message_block.chunk_flag {
                        parse_record_index += 1;
                    }

                    NDEFParseState::MessageHeader
                }
            }
        };
    }

    return Ok(parse_result);
}

#[derive(Debug)]
pub struct MessageBlock {
    //pub message_begin: bool,
    pub message_end: bool,
    pub chunk_flag: bool,
    pub short_record: bool,
    pub id_length: bool,
    //pub type_name_format: u8
}

pub fn parse_ndef_message_block(byte: u8) -> MessageBlock {
    return MessageBlock {
        //message_begin: (byte & 0b10000000) != 0,
        message_end: (byte & 0b01000000) != 0,
        chunk_flag: (byte & 0b00100000) != 0,
        short_record: (byte & 0b00010000) != 0,
        id_length: (byte & 0b00001000) != 0,
        //type_name_format: byte & 0b00000111
    }
}

pub fn get_uri_protocol(identifier: u8) -> &'static str {
    match identifier {
        0x00 => "",
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
