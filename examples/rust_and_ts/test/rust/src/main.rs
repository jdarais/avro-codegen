use rust_and_ts::org::testorg::test::{RecordWithUnion, MyEnum, TestRecord};
use rust_and_ts::_unions::Union4;
use rust_and_ts::RecordWithRef;

fn test_record_with_union() -> anyhow::Result<()> {
    let record = RecordWithUnion {
        some_int: 25,
        test: Union4::Variant3(String::from("hello")),
        enum_field: MyEnum::Yabba,
    };

    // Serialize the record
    let mut buffer: Vec<u8> = Vec::new();
    let mut writer = RecordWithUnion::writer(&mut buffer);
    writer.append_ser(&record)?;
    writer.flush()?;
    drop(writer);

    // Read it back
    let mut reader = RecordWithUnion::reader(buffer.as_slice())?;
    let parsed_opt = RecordWithUnion::read_next(&mut reader)?;
    let parsed = parsed_opt.unwrap();

    assert_eq!(&record, &parsed);

    Ok(())
}

fn test_record_with_ref() -> anyhow::Result<()> {
    let record = RecordWithRef {
        some_int: 45,
        test: Some(TestRecord {
            somefield: String::from("hi")
        })
    };

    let mut buffer: Vec<u8> = Vec::new();
    record.write_single(&mut buffer)?;

    let parsed_opt = RecordWithRef::read_single(buffer.as_slice())?;
    let parsed = parsed_opt.unwrap();

    assert_eq!(&record, &parsed);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    test_record_with_union()?;
    test_record_with_ref()?;

    Ok(())
}
