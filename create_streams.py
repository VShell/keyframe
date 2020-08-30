import base64
import io
import json
import os
import pathlib
import secrets
import string
import subprocess
import sqlite3
import tempfile
import toml

streamsPath = os.environ['streamsPath']
domain = os.environ['domain']

with open(streamsPath) as f:
    streams = json.load(f)

keyframe_streams_db = sqlite3.connect('/var/lib/keyframe/streams.db')
keyframe_streams_db.row_factory = sqlite3.Row
keyframe_streams_db.execute('PRAGMA journal_mode=MEMORY')
ingestd_streams_db = sqlite3.connect('/var/lib/ingestd/streams.db')
ingestd_streams_db.row_factory = sqlite3.Row
#ingestd_streams_db.execute('PRAGMA journal_mode=MEMORY')

ensure_jids = ""
remove_jids = ""

with open('/var/lib/ingestd/ingestd-srt.toml') as f:
    config = toml.load(f)
    blake3_key = base64.b64decode(config['secret'].encode())

blake3_key_file = tempfile.NamedTemporaryFile(delete=False)
blake3_key_file.write(blake3_key)

# For each stream in the list, ensure it has a room, XMPP account, and is in the database - if it's new, send an email
for (name, config) in streams.items():
    jid = config.get('jid') or f'{name}@{domain}'
    ensure_jids += f'{name}@streamchat.{domain}\t{jid}\n'

    try:
        keyframe_streams_cursor = keyframe_streams_db.execute('INSERT INTO streams (mpd_url) VALUES (?)', (f'https://{domain}/stream/{name}.mpd',))
    except sqlite3.IntegrityError:
        continue

    ingestd_token = secrets.token_urlsafe()
    keyframe_streams_db.execute('INSERT INTO ingestd_tokens (stream_id, token) VALUES (?, ?)', (keyframe_streams_cursor.lastrowid, ingestd_token))
    ingestd_cursor = ingestd_streams_db.execute('INSERT INTO streams (active, notify_url, token) VALUES (TRUE, ?, ?)', (f'https://{domain}/api/v1/ingestd-notify', ingestd_token))

    srt_streamid = f'#!::u={ingestd_cursor.lastrowid}'
    with tempfile.NamedTemporaryFile(delete=False) as blake3_file:
        blake3_file.write(srt_streamid.encode())
        blake3_filename = blake3_file.name
    blake3_key_file.seek(0)
    with subprocess.Popen(['b3sum', '--keyed', '--no-names', blake3_filename], stdin=blake3_key_file, stdout=subprocess.PIPE) as b3sum:
        srt_passphrase = b3sum.stdout.read().strip()

    xmpp_password = ''.join(secrets.choice(string.ascii_letters + string.digits) for i in range(12))
    if config.get('jid') is None:
        subprocess.run(['prosodyctl', 'register', name, domain, xmpp_password], stdin=subprocess.DEVNULL, check=True)

    email = f'Subject: New stream at {domain}\n\n'
    email += f'Stream URL: https://{domain}/stream/{name}\n'
    email += f'Stream SRT URL: srt://ingestd.{domain}:3800?streamid={srt_streamid}&passphrase={srt_passphrase.decode()}\n'
    if config.get('jid') is None:
        email += f'\nXMPP username: {name}@{domain}\n'
        email += f'XMPP password: {xmpp_password}\n'
    with tempfile.TemporaryFile() as email_file:
        email_file.write(email.encode())
        email_file.seek(0)
        subprocess.run(['sendmail', config['email']], stdin=email_file)

# For each stream in the database that isn't in the list, remove it from the database and remove its room
mpd_urls = {f'https://{domain}/stream/{name}.mpd': name for name in streams.keys()}
for row in keyframe_streams_db.execute('SELECT id, mpd_url FROM streams'):
    if row['mpd_url'] in mpd_urls:
        continue

    keyframe_streams_db.execute('DELETE FROM streams WHERE id = ?', row['id'])
    remove_jids += f'{mpd_urls[row["mpd_url"]]}@streamchat.{domain}\n'

keyframe_streams_db.commit()
ingestd_streams_db.commit()

# Reload ingestd-srt
if subprocess.run(['systemctl', 'is-active', 'ingestd-srt', '--quiet'], stdin=subprocess.DEVNULL).returncode == 0:
    subprocess.run(['systemctl', 'reload', 'ingestd-srt'], stdin=subprocess.DEVNULL)

xmpp_admin_password = pathlib.Path('/var/lib/keyframe/stream-muc-manager/xmpp-password').read_text().strip()

print(ensure_jids.encode())
print(remove_jids.encode())

# Run stream-muc-manager
with tempfile.NamedTemporaryFile(delete=False) as ensure_jids_file:
    ensure_jids_file.write(ensure_jids.encode())
    ensure_jids_filename = ensure_jids_file.name

with tempfile.NamedTemporaryFile(delete=False) as remove_jids_file:
    remove_jids_file.write(remove_jids.encode())
    remove_jids_filename = remove_jids_file.name

subprocess.run(['stream-muc-manager', '-jid', f'stream-muc-manager@streamadmin.{domain}', '-password', xmpp_admin_password, '-ensure', ensure_jids_filename, '-remove', remove_jids_filename], stdin=subprocess.DEVNULL, check=True)
