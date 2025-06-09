The Azure MCP Server seamlessly integrates with your host operating system's
authentication mechanisms, making it super easy to get started! We use Azure
Identity under the hood via
[DefaultAzureCredential](https://learn.microsoft.com/dotnet/azure/sdk/authentication/credential-chains?tabs=dac),
which tries these credentials in order:

1. Environment Variables (`EnvironmentCredential`) - Perfect for CI/CD pipelines
1. Shared Token Cache (`SharedTokenCacheCredential`) - Uses cached tokens from
   other tools
1. Visual Studio (`VisualStudioCredential`) - Uses your Visual Studio
   credentials
1. Azure CLI (`AzureCliCredential`) - Uses your existing Azure CLI login
1. Azure PowerShell (`AzurePowerShellCredential`) - Uses your Az PowerShell
   login
1. Azure Developer CLI (`AzureDeveloperCliCredential`) - Uses your azd login
   Interactive Browser (`InteractiveBrowserCredential`) - Falls back to
   browser-based login if needed.

If you're already logged in through any of these methods, the Azure MCP Server
will automatically use those credentials. Ensure that you have the correct
authorization permissions in Azure (e.g. read access to your Storage account)
via RBAC (Role-Based Access Control).

By default, the Azure MCP Server excludes production credentials like Managed
Identity and Workload Identity. To enable these credentials, set the
`enable_production_credentials` setting to `true` in your Zed `settings.json`.

```json
{
  "context_servers": {
    "azure": {
      "settings": {
        "enable_production_credentials": true
      }
    }
  }
}
```
