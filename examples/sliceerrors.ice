
module ErrorModule1
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

module ErrorModule2
{
    struct Foo
    {
        Bar out_of_scope;
    }
}

module ClashWithTesting {}
