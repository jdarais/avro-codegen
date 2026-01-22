import { type RecordWithTypes, RECORD_WITH_TYPES_SCHEMA } from "rust-and-ts/org/testorg";

describe("Record with types", () => {
    it("serializes and deserializes", () => {
        const record: RecordWithTypes = {
            nullableBoolean: null,
            intValue: 7,
            longValue: 95,
            floatValue: 3.5,
            doubleValue: 9.09,
            bytesValue: Buffer.from([9, 12]),
            stringValue: "hello",
            unionValue: { float: 8.4 },
            recordValue: { lengthM: 5.1, widthM: 2.4 },
            enumValue: "spades",
            fixedValue: Buffer.from([45, 22, 91, 0, 1, 128, 255, 191]),
            numbers: [7, 5, 4, 1],
            ages: {"Paul": 27, "Susan": 29},
            decimalValue: Buffer.from([1, 2, 3]),
            bigDecimalValue: Buffer.from([1, 2, 3]),
            durationValue: Buffer.from([0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1]),
            uuidValue: Buffer.from([234, 195, 1, 7, 94, 48, 29, 3, 85, 61, 42, 44, 32, 99, 108, 33])
        };

        const serialized = RECORD_WITH_TYPES_SCHEMA.toBuffer(record);
        const deserialized = RECORD_WITH_TYPES_SCHEMA.fromBuffer(serialized);

        expect(RECORD_WITH_TYPES_SCHEMA.compare(deserialized, record)).toBeTruthy();
    });
});
