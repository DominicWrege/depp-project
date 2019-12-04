$input = $args[0]

$raw = get-content $input -Raw
$regex="(?<![0-9\-])[0]*((10000)|([0-9]{0,3}[1-9])|([0-9]{0,2}[1-9][0-9]?)|([0-9]?[1-9][0-9]{0,2})|([1-9][0-9]{0,3}))([\.\,][0-9]*)?([\s\n\r]+|[a-zA-Z])[a-zA-Z]*"
($raw | Select-String -AllMatches -Pattern $regex).Matches | ForEach-Object{
	$current=$_.Value
	$current -replace "\s",' '
}
