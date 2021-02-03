
// This is the worst slice file I've ever seen.

module hello
{
    /// this struct
    /// doesn't do anything
    struct  myStruct
    {
        int something;                      // TODO
        tag(1) float    somethingElse123;
        string? custom_Thing0;
        tag(256)   MyType? custom_stuff;
        Service        newKeyword;
        AnyClass?        WowPrettyCool;

        sequence<varuint> mySeq66_;
        dictionary<Foo?, Bar::Baz> custom_seq;
    };

    /* This is a test */
    module inner::nested
    {
        exception /** Another test!*/ OhNo
        {
            myStruct whatHappened;
        }

        interface MyOps
        {
            void noOp();
            int opInt(int i);

            void tag_stuff1(tag(99) sequence<int> is, bool myBool);
            (int i, tag(1) string s) tupleThings(long l);
        }
    }

    interface useless{};

    unchecked enum myEnum : varint
    {
        // Blah blah
    }
}
