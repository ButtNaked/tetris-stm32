
use phf::phf_map;
use super::rtt_print;



pub fn parse_n_answer(buf : &mut[u8; 0x40]) -> usize {
    #[repr(packed)]
    #[repr(C)]
    struct CmdIn {
        sign : u8,
        op : Operation,
        name_sz : u8,
        value_sz : u8,
        payload : [u8; 0x40 - 4],
    }

    let cmd: &mut CmdIn = unsafe { core::mem::transmute(buf.as_mut_ptr()) };
    let name: &str = unsafe { core::str::from_utf8_unchecked(&cmd.payload[0 .. cmd.name_sz as usize])};
    match dispatch_blocking(cmd.op, name) {
        Ok(RegType::u32V(value)) => {
            cmd.payload[cmd.name_sz as usize + 0] = value as u8;
            cmd.payload[cmd.name_sz as usize + 1] = (value >> 8) as u8;
            cmd.payload[cmd.name_sz as usize + 2] = (value >> 16) as u8;
            cmd.payload[cmd.name_sz as usize + 3] = (value >> 24) as u8;
            cmd.value_sz = 4;
        }
        Err(Error::Ok) => {
            cmd.value_sz = 0;
        }
        _ => unimplemented!(),
    };

    (4 + cmd.name_sz + cmd.value_sz) as usize
}


#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Read = 0,
    Write,
    Erase,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum RegType {
    u32V(u32),
    u8V(u8),
}

pub type CmdResult = Result<RegType, Error>;

pub fn dispatch_blocking(op : Operation, name : &str) -> CmdResult {

    let e_num = STR_TO_ENUM.get(name).ok_or(Error::NoSuch)?;
    let e = &REGISTRY[*e_num as usize];

    match (op, e) {
        (Operation::Erase, RegistryEntityType::Section) => {
            rtt_print!("Erasing section {}", name);
            Err(Error::Ok)
        }
        (Operation::Read, RegistryEntityType::Register(_r)) => {
            rtt_print!("Reading reg {}", name);
            Ok(RegType::u32V(42))
        }
        (Operation::Write, RegistryEntityType::Register(_r)) => {
            rtt_print!("Writing reg {}", name);
            Err(Error::Ok)
        }
        (_, _) => return Err(Error::WrongOperation),
    }
}

#[derive(Clone, Copy, Debug)]
enum RegistryEntityType {
    Section,
    Register(Register),
}

#[derive(Clone, Copy, Debug)]
struct Register {
    dummy : u32,
}

const REGISTRY : [RegistryEntityType; RegistryEntityNum::Max as usize] = [
    RegistryEntityType::Section,
    RegistryEntityType::Register(Register { dummy : 0 }),
    RegistryEntityType::Register(Register { dummy : 1 }),
    RegistryEntityType::Register(Register { dummy : 2 }),
];

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
enum RegistryEntityNum {
    test,
    test_echo,
    test_reg,
    test_num,
    Max,
}

const STR_TO_ENUM : phf::Map<&'static str, RegistryEntityNum> = phf_map! {
    "test"      => RegistryEntityNum::test,
    "test/echo" => RegistryEntityNum::test_echo,
    "test/reg"  => RegistryEntityNum::test_reg,
    "test/num"  => RegistryEntityNum::test_num,
};

pub fn str_to_enum(s : &'static str) {
    let c = STR_TO_ENUM.get(s);
    rtt_print!("{:?}",  &c);
}

pub enum Error {
    Ok = 0,
    WrongOperation,
    BadFormat,
    NoSuch,
    NonWriteable,
    Locked,
    EraseNeeded,
}