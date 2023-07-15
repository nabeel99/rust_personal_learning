#!/bin/bash
#set -x prints out the commands as they are executed
set -x
#-e used to exit if any one the command in a chain errors
# -o pipefail used to exit if any command in a pipe fail
set -eo pipefail


#check if dependency is installed 
if ! [ -x "$(command -v psql)" ];then
echo >&2 "Error : psql is not installed"
exit 1
fi
#bugged
if ! [ -x "$(command -v sqlx)" ]; then
echo >&2 "Error: sqlx is not installed."
echo >&2 "Use:"
echo >&2 " cargo install --version='~0.6' sqlx-cli \
--no-default-features --features rustls,postgres"
echo >&2 "to install it."
exit 1
fi
# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER=${POSTGRES_USER:=postgress}
# Check if a custom password has been set, otherwise default to 'password
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"

# Check if a custom database name has been set, otherwise default to 'newsletter
DB_NAME="${POSTGRES_DB:=newsletter}"
#Check if a custom port has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

#Launch Postgres using Docker
#docker run -d runs the image in the background and prints its container id
#invalid refrence error : extra tab space was there
if [[ -z "${SKIP_DOCKER}" ]]
then
docker run \
-e POSTGRES_USER=${DB_USER} \
-e POSTGRES_PASSWORD=${DB_PASSWORD} \
-e POSTGRES_DB=${DB_NAME} \
-p "${DB_PORT}":5432 \
-d postgres \
postgres -N 1000
fi
# ^ Increased maximum number of connections for testing purposes


# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q';
do >&2 echo "Postgress is still unavailable -sleeping"
sleep 1
done
>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

#POSTGRES DATABASE Connection String
#postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
 export DATABASE_URL

sqlx database create
sqlx migrate run
>&2 echo "Postgres has been migrated, ready to go!"
# Assuming you used the default parameters to launch Postgres in Docker!


