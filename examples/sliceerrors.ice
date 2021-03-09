
module Module1
{
    struct Foo
    {
        int duplicate;
        string duplicate;
    }

    struct Bar
    {
        faketype ft;
    }
}

module Module2
{
    struct Foo
    {
        Bar out_of_scope;
    }
}
