param([Parameter(Mandatory)][string]$Path)
$ErrorActionPreference = 'Stop'
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.OpenDatabase((Resolve-Path -LiteralPath $Path).Path, 0)
function Read-Rows([string]$Sql, [int]$Columns) {
  $view = $database.OpenView($Sql)
  $null = $view.Execute()
  try {
    while ($record = $view.Fetch()) {
      $values = @(for ($column = 1; $column -le $Columns; $column++) { $record.StringData($column) })
      [pscustomobject]@{ Values = $values }
    }
  } finally { $null = $view.Close() }
}
$properties = @{}
foreach ($row in (Read-Rows 'SELECT `Property`, `Value` FROM `Property`' 2)) { $properties[$row.Values[0]] = $row.Values[1] }
if ($properties.ProductName -ne 'Raygun') { throw 'MSI product name must be Raygun' }
$stock = '229BB552-5EFD-479A-BA86-8CB6D926390C'
if ($properties.UpgradeCode.Trim('{}') -eq $stock) { throw 'MSI still shares the stock LightCraft upgrade identity' }
if ($properties.UpgradeCode.Trim('{}') -ne 'BCDAA01E-634D-4E76-BF91-82085B21C3A5') { throw 'MSI changed the permanent Raygun upgrade identity' }
foreach ($row in (Read-Rows 'SELECT `ComponentId` FROM `Component`' 1)) {
  if ($row.Values[0].Trim('{}') -eq '8A1D3A39-F7C1-47B1-97AE-40C80BF82F4A') { throw 'MSI reuses the stock application component identity' }
}
foreach ($row in (Read-Rows 'SELECT `UpgradeCode` FROM `Upgrade`' 1)) {
  if ($row.Values[0].Trim('{}') -eq $stock) { throw 'MSI must not upgrade or remove stock LightCraft' }
}
foreach ($row in (Read-Rows 'SELECT `Key` FROM `Registry`' 1)) {
  if ($row.Values[0] -match '(?i)LightCraft') { throw "MSI writes a stock application registry key: $($row.Values[0])" }
}
$files = @(Read-Rows 'SELECT `FileName` FROM `File`' 1 | ForEach-Object { ($_.Values[0] -split '\|')[-1] })
foreach ($required in 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'LICENSE-LGPL-2.1', 'NOTICE', 'OFL-Inter.txt') {
  if ($files -notcontains $required) { throw "MSI is missing $required" }
}
Write-Output "ok: compiled MSI $($properties.ProductVersion), independent identity and required notices"
