using System.Collections.Generic;

namespace Yggdrasil.Test;

public class CustomStrategyReturningTrue : IStrategy
{
    public CustomStrategyReturningTrue(string name)
    {
        Name = name;
    }

    public string Name { get; private set; }

    public bool IsEnabled(Dictionary<string, string> parameters, Context context)
    {
        return true;
    }
}