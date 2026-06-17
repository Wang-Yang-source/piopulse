$ErrorActionPreference = 'Stop'

$packageName = 'piopulse'
$fileType = 'exe'
$url64 = 'https://github.com/Wang-Yang-source/piopulse/releases/download/v0.2.2/piopulse-windows-x86_64.zip'
$checksum64 = 'INSERT_WINDOWS_X86_64_SHA256_HERE'
$checksumType64 = 'sha256'

$packageArgs = @{
  packageName   = $packageName
  unzipLocation = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64= $checksumType64
}

Install-ChocolateyZipPackage @packageArgs
