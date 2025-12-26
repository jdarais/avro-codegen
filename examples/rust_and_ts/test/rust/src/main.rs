
fn main() {

}

mod tests {
    use rust_and_ts::org::testorg::test::{RecordWithUnion, MyEnum, TestRecord};
    use rust_and_ts::org::testorg::InternallyDefinedRecord;
    use rust_and_ts::_unions::Union4;
    use rust_and_ts::RecordWithRef;

    #[test]
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
        let parsed = reader.next().unwrap()?;

        assert_eq!(&record, &parsed);

        Ok(())
    }

    #[test]
    fn test_record_with_ref() -> anyhow::Result<()> {
        let record = RecordWithRef {
            some_int: 45,
            test: Some(TestRecord {
                somefield: String::from("hi")
            })
        };

        let mut buffer: Vec<u8> = Vec::new();
        let mut writer = RecordWithRef::writer(&mut buffer);
        writer.append_ser(&record)?;
        drop(writer);

        let mut reader = RecordWithRef::reader(buffer.as_slice())?;
        let parsed = reader.next().unwrap()?;

        assert_eq!(&record, &parsed);

        Ok(())
    }
}