[AttributeUsage(AttributeTargets.Assembly)]
public class YggdrasilCoreVersionAttribute : System.Attribute
{
    public string Version { get; }

    public YggdrasilCoreVersionAttribute(string version)
    {
        Version = version;
    }
}