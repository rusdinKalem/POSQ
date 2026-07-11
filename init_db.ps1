$pgBin = "$env:USERPROFILE\scoop\apps\postgresql\current\bin"
$pgData = "$env:USERPROFILE\scoop\apps\postgresql\current\data"

Write-Host "Starting PostgreSQL Server..."
Start-Process "$pgBin\pg_ctl.exe" -ArgumentList "-D", $pgData, "-l", "$pgData\server.log", "start" -WindowStyle Hidden

Write-Host "Waiting 3 seconds for server to start..."
Start-Sleep -Seconds 3

Write-Host "Creating user and database..."
& "$pgBin\psql.exe" -U postgres -c "CREATE USER pos_app WITH PASSWORD 'pos_app_dev';"
& "$pgBin\psql.exe" -U postgres -c "CREATE DATABASE pos_local OWNER pos_app;"

Write-Host "PostgreSQL Setup Complete!"
