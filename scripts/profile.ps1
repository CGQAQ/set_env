# ----------------------------------SET_ENV_BEG
# ----------------------------------SET_ENV_UFNS_BEG
function setenv_set([string]$name, [string]$value) {
    New-Item env:\$name -Value $value -Force | Out-Null
}

function setenv_unset([string]$name) {
    if (Test-Path env:\$name) {
        Clear-Item env:\$name | Out-Null
    }
}

function setenv_set_if_not_exist([string]$name, [string]$value) {
    if ( -Not (Test-Path env:\$name) ) {
        setenv_set $name $value
    }
}

function setenv_prepend([string]$name, [string]$value) {

    [Environment]::SetEnvironmentVariable($name, "$value;$(Get-Content -Path env:$name)", "Process")
}

function setenv_append([string]$name, [string]$value) {
    [Environment]::SetEnvironmentVariable($name, "$(Get-Content -Path env:$name);$value", "Process")
}

function setenv_path_append([string]$value) {
    $env:Path += ";$value"
}

function setenv_path_prepend([string]$value) {
    $env:Path = "$value;$env:Path"
}

function setenv_path_append_if_not_contains([string]$value) {
    $paths = $env:Path.Split(";");
    if ($paths -notcontains "$value") {
        setenv_path_append $value
    }
}

function setenv_path_prepend_if_not_contains([string]$value) {
    $paths = $env:Path.Split(";");
    if ($paths -notcontains "$value") {
        setenv_path_prepend $value
    }
}
# ----------------------------------SET_ENV_UFNS_END
# ----------------------------------SET_ENV_DEFS_BEG
# ----------------------------------SET_ENV_DEFS_END
# ----------------------------------SET_ENV_END