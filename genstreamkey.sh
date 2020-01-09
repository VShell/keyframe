#!@bash@

if [[ -z $1 ]]; then
  echo usage: genstreamkey USERNAME >> /dev/stderr
  exit 1
fi

key=$(tr -dc abcdefghijklmnopqrstuvwxyz < /dev/urandom | fold -w 20 | head -n1)

ok=0
while IFS= read -r line; do
  username=$(printf '%s' $line | cut -d: -f1)
  if [[ $username == $1 ]]; then
    printf '%s:%s\n' $username $key
    ok=1
  else
    printf '%s\n' $line
  fi
done < /var/lib/rtmpauth/users > /var/lib/rtmpauth/users.new

if [[ $ok != 1 ]]; then
  echo no such user >> /dev/stderr
  exit 1
else
  mv /var/lib/rtmpauth/users{.new,}
  echo $key
  exit 0
fi
