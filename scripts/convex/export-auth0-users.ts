import {
  buildLifecycle,
  createPostgresClient,
  sanitizeJson,
  writeJsonFile,
} from "./shared";

async function main() {
  const client = createPostgresClient();
  await client.connect();

  try {
    const result = await client.query(`
      select
        id,
        email,
        password_hash,
        email_verified,
        settings,
        totp_enabled,
        created_at,
        updated_at
      from users
      where email is not null
      order by created_at asc
    `);

    const exportedUsers = result.rows.map((row: Record<string, unknown>) => {
      const lifecycle = buildLifecycle(row);
      const passwordHash =
        typeof row.password_hash === "string" ? row.password_hash : undefined;

      return {
        user_id: row.id,
        email: typeof row.email === "string" ? row.email.toLowerCase() : undefined,
        email_verified: Boolean(row.email_verified),
        app_metadata: {
          legacy_user_id: row.id,
          migrated_from: "postgres",
          migrated_at: lifecycle.updatedAt,
          totp_enabled: Boolean(row.totp_enabled),
        },
        user_metadata: sanitizeJson(row.settings) ?? {},
        ...(passwordHash
          ? {
              custom_password_hash: {
                algorithm: "bcrypt",
                hash: {
                  value: passwordHash,
                },
              },
            }
          : {}),
      };
    });

    const outputPath = await writeJsonFile(
      process.env.AUTH0_IMPORT_OUTPUT ?? "tmp/auth0-users.json",
      exportedUsers,
    );

    console.log(`Exported ${exportedUsers.length} users to ${outputPath}`);
  } finally {
    await client.end();
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
