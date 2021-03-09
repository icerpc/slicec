
module Module1
{
    struct MyStruct
    {
        int i;
        string s;
        Time time;
        Clock clock;
    }

    struct Time
    {
        int minutes;
        int hours;
    }

    interface Clock {}
}

module Module2
{
    struct BaseStruct {}

    module Outer
    {
        struct OuterStruct {}

        module Inner
        {
            struct InnerStruct {}

            struct ScopeTest {
                BaseStruct s1;
                OuterStruct s2;
            }
        }

        struct ScopeTest {
            BaseStruct s1;
            Inner::InnerStruct s2;
        }
    }

    struct ScopeTest {
        Outer::OuterStruct s1;
        Outer::Inner::InnerStruct s2;
    }
}

module Module3
{
    struct Foo
    {
        Module2::BaseStruct bs;
        ::Module1::Clock clock;
    }
}
