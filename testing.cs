//Start of file
namespace Module1
{
    struct MyStruct
    {
        int i;
        string s;
        Module1.Time time;
        Module1.Clock clock;
    }

    struct Time
    {
        int minutes;
        int hours;
    }

    interface Clock
    {
    }

}

namespace Module2
{
    struct BaseStruct
    {
    }

    namespace Outer
    {
        struct OuterStruct
        {
        }

        namespace Inner
        {
            struct InnerStruct
            {
            }

            struct ScopeTest
            {
                Module2.BaseStruct s1;
                Module2.Outer.OuterStruct s2;
            }

        }

        struct ScopeTest
        {
            Module2.BaseStruct s1;
            Module2.Outer.Inner.InnerStruct s2;
        }

    }

    struct ScopeTest
    {
        Module2.Outer.OuterStruct s1;
        Module2.Outer.Inner.InnerStruct s2;
    }

}

namespace Module3
{
    struct Foo
    {
        Module2.BaseStruct bs;
        Module1.Clock clock;
    }

}

namespace ClashWithTesting
{
}

//End of file
