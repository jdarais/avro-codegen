use rust_and_ts::org::testorg::test::{RecordWithUnion, MyEnum};
use rust_and_ts::_unions::Union4;

fn test_record_with_union() -> anyhow::Result<()> {
    let record = RecordWithUnion {
        some_int: 25,
        test: Union4::Variant3(String::from("hello")),
        enum_field: MyEnum::Yabba,
    };

    // Serialize the record
    let mut buffer: Vec<u8> = Vec::new();
    record.write_single(&mut buffer)?;

    // Read it back
    let parsed = RecordWithUnion::read_single(buffer.as_slice()).unwrap()?;

    assert_eq!(&record, &parsed);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    test_record_with_union()?;

    Ok(())
}
