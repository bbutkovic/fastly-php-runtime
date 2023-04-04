use std::{
    cell::RefCell,
    io::{stdin, Read},
};

use bytes::{buf::Reader, Buf, Bytes};

pub struct CodeReaderHandle(Reader<Bytes>, usize);

thread_local! {
    static INSTANCE: RefCell<Option<CodeReaderHandle>> = RefCell::new(None);
}

impl CodeReaderHandle {
    pub fn load_from_stdin() -> anyhow::Result<()> {
        let input: Bytes = stdin().bytes().map(|b| b.unwrap()).collect();
        let size = input.len();
        let reader = input.reader();

        INSTANCE.with(|code_reader| {
            *code_reader.borrow_mut() = Some(CodeReaderHandle(reader, size));
        });

        Ok(())
    }

    pub fn read(buf: &mut [u8]) -> anyhow::Result<usize> {
        INSTANCE.with(|code_reader| {
            let mut reader = (*code_reader.borrow_mut()).take().unwrap();
            let res = reader.0.read(buf);

            *code_reader.borrow_mut() = Some(reader);
            res.map_err(anyhow::Error::from)
        })
    }

    pub fn size() -> usize {
        INSTANCE.with(|code_reader| {
            let reader = (*code_reader.borrow_mut()).take().unwrap();
            let res = reader.1;

            *code_reader.borrow_mut() = Some(reader);
            res
        })
    }
}
