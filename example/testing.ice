
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
    module Outer
    {
        module Inner
        {
            struct InnerStruct {
                BaseStruct s1;
                OuterStruct s2;
            }
        }

        struct OuterStruct {
            BaseStruct s1;
            Inner::InnerStruct s2;
        }
    }

    struct BaseStruct {
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
