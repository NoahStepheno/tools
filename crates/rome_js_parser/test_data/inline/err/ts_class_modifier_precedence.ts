class Base { base1; base2; base3; base4;}
abstract class Test extends Base {
    // Accessibility
    readonly private a: string;
    override protected base1;
    static private b: string;
    abstract protected d: string;
    // Static
    readonly static c: string;
    override static base2: string;
    // abstract
    override abstract base3: string;
    // override
    readonly override base4: string;
}
