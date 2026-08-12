import { RECORD_WITH_TYPES_SCHEMA } from "sample-schemas/org/testorg";
import { Buffer } from "buffer";

describe("Record with types", () => {
    it("serializes and deserializes", () => {
        const record = {
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
            durationValue: { months: 3, days: 7, millis: 1000 },
            uuidValue: "6ec0bd7f-11c0-43da-975e-2a8ad9ebae0b"
        };

        const serialized = RECORD_WITH_TYPES_SCHEMA.toBuffer(record);
        const deserialized = RECORD_WITH_TYPES_SCHEMA.fromBuffer(serialized);

        expect(RECORD_WITH_TYPES_SCHEMA.compare(deserialized, record)).toBeTruthy();
    });
});
