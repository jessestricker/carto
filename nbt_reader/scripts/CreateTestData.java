import java.io.ByteArrayOutputStream;
import java.io.DataOutput;
import java.io.DataOutputStream;
import java.io.IOException;
import java.util.StringJoiner;

public class CreateTestData {
    public static void main(String[] args) throws IOException {
        printDataOutput("read_byte", "-64",
            (dataOutput) -> dataOutput.writeByte(-64));
        printDataOutput("read_short", "-16384",
            (dataOutput) -> dataOutput.writeShort(-16384));
        printDataOutput("read_unsigned_short", "32767",
            (dataOutput) -> dataOutput.writeShort(32767));
        printDataOutput("read_int", "-1073741824",
            (dataOutput) -> dataOutput.writeInt(-1073741824));
        printDataOutput("read_long", "-4611686018427387904",
            (dataOutput) -> dataOutput.writeLong(-4611686018427387904L));

        // U+0041, A, Latin Capital Letter A, 1 byte
        // U+03BC, μ, Greek Small Letter Mu, 2 bytes
        // U+0000, NUL, 2 bytes
        // U+121F, ሟ, Ethiopic Syllable Mwa, 3 bytes
        printDataOutput("read_utf", "\"\\u{0041}\\u{03BC}\\u{0000}\\u{121F}\"",
            (dataOutput) -> dataOutput.writeUTF("\u0041\u03BC\u0000\u121F"));
    }

    private static void printDataOutput(String name, String expected, Handler handler) throws IOException {
        var baos = new ByteArrayOutputStream();
        var dos = new DataOutputStream(baos);
        handler.handle(dos);
        var bytes = baos.toByteArray();

        var bytesLiteral = new StringJoiner(", ", "[", "]");
        for (var b : bytes) {
            bytesLiteral.add("%#x".formatted(b));
        }
        System.out.printf("#[test]\n");
        System.out.printf("fn %s() {\n", name);
        System.out.printf("    pub const DATA: [u8; %s] = %s;\n", bytes.length, bytesLiteral.toString());
        System.out.printf("    let mut reader = Cursor::new(&DATA);\n");
        System.out.printf("    assert_eq!(%s, reader.%s().unwrap());\n", expected, name);
        System.out.printf("    assert_eq!(DATA.len(), reader.position() as usize);\n");
        System.out.printf("}\n");
    }

    @FunctionalInterface
    private interface Handler {
        void handle(DataOutput dataOutput) throws IOException;
    }
}
