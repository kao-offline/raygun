# Regression guard: Raygun must never upgrade or overwrite a stock LightCraft installation.
$ErrorActionPreference = 'Stop'
[xml]$source = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'raygun.wxs') -Raw
$ns = [System.Xml.XmlNamespaceManager]::new($source.NameTable)
$ns.AddNamespace('w', 'http://wixtoolset.org/schemas/v4/wxs')
$package = $source.SelectSingleNode('//w:Package', $ns)
if ($package.Name -ne 'Raygun' -or $package.UpgradeCode -eq '229BB552-5EFD-479A-BA86-8CB6D926390C') {
    throw 'Raygun must have a separate product name and upgrade identity'
}
$null = [guid]::Parse($package.UpgradeCode)
if ($package.UpgradeCode -ne 'BCDAA01E-634D-4E76-BF91-82085B21C3A5') { throw 'Raygun must retain its permanent upgrade identity for future updates' }
$folder = $source.SelectSingleNode('//w:Directory[@Id="INSTALLFOLDER"]', $ns)
if ($folder.Name -ne 'Raygun') { throw 'Default install folder must be Raygun' }
foreach ($registry in $source.SelectNodes('//w:RegistrySearch | //w:RegistryValue', $ns)) {
    if ($registry.Key -match '(?i)LightCraft') { throw "Stock application registry collision: $($registry.Key)" }
}
$previous = $source.SelectSingleNode('//w:RegistrySearch[@Id="PreviousInstallFolder"]', $ns)
if ($previous.Key -ne 'Software\Raygun\Installer') { throw 'Only the separate Raygun installer may supply a previous folder' }
$app = $source.SelectSingleNode('//w:Component[@Id="RaygunApp"]', $ns)
if ($app.Guid -eq '8A1D3A39-F7C1-47B1-97AE-40C80BF82F4A') { throw 'Stock application component identity must not be reused' }
if ($app.Guid -ne '*') { $null = [guid]::Parse($app.Guid) }
Write-Output 'ok: independent Raygun installer identity and registry paths'
