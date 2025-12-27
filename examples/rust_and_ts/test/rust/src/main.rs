
fn main() {

}

mod tests {
    use std::collections::HashMap;

    use rust_and_ts::org::testorg::test::{RecordWithUnion, MyEnum, TestRecord};
    use rust_and_ts::org::testorg::{InternallyDefinedRecord, RecordWithTypes, LongWord, Suit, Size};
    use rust_and_ts::_unions::{Union3, Union4};
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
        let mut writer = RecordWithUnion::writer(&mut buffer)?;
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
        let mut writer = RecordWithRef::writer(&mut buffer)?;
        writer.append_ser(&record)?;
        drop(writer);

        let mut reader = RecordWithRef::reader(buffer.as_slice())?;
        let parsed = reader.next().unwrap()?;

        assert_eq!(&record, &parsed);

        Ok(())
    }

    #[test]
    fn test_record_with_types() -> anyhow::Result<()> {
        let record = RecordWithTypes {
            nullable_boolean: Some(false),
            int_value: 75,
            long_value: 0xaaaaaaaaaaa,
            float_value: 5.4,
            double_value: 7.6,
            bytes_value: vec![b'4'].into(),
            string_value: String::from("hello"),
            fixed_value: LongWord([12, 1, 56, 43, 111, 204, 109, 3]),
            enum_value: Suit::Spades,
            union_value: Union3::Variant2(704.4),
            record_value: Size {
                length_m: 4.5,
                width_m: 1.2
            },
            numbers: vec![1, 2, 3, 4, 5, 6789],
            ages: vec![
                (String::from("Paul"), 42),
                (String::from("Lisa"), 45),
                (String::from("Carol"), 13),
                (String::from("Jake"), 7)
            ].into_iter().collect::<HashMap<String, i32>>(),
            decimal_value: apache_avro::Decimal::from(&[251, 155]),
            big_decimal_value: "45.7".parse()?
        };

        let mut buffer: Vec<u8> = Vec::new();
        let mut writer = RecordWithTypes::writer(&mut buffer)?;
        writer.append_ser(&record)?;
        drop(writer);

        let mut reader = RecordWithTypes::reader(buffer.as_slice())?;
        let parsed = reader.next().unwrap()?;

        assert_eq!(&record, &parsed);

        Ok(())
    }
}