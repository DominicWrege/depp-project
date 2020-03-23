#! /bin/bash

read -a input
length=${#input}
echo "Ausgabe: $input"
reverse=""
for i in $(seq 1 $length )
do
	reverse+=${input:$length-i:1}
done
echo "Ausgabe umgekehrt: $reverse"
