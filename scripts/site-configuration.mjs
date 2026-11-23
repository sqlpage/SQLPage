function schemaType(schema) {
  if (schema.enum)
    return schema.enum.map((value) => JSON.stringify(value)).join(" | ");
  if (schema.anyOf) return schema.anyOf.map(schemaType).join(" or ");
  const types = Array.isArray(schema.type) ? schema.type : [schema.type];
  return types
    .map((type) => {
      if (type === "array") return `${schemaType(schema.items)}[]`;
      if (type === "integer" || type === "number") {
        if (schema.minimum !== undefined && schema.maximum !== undefined)
          return `${type} (${schema.minimum} to ${schema.maximum})`;
        if (schema.minimum !== undefined)
          return `${type} (≥ ${schema.minimum})`;
        if (schema.maximum !== undefined)
          return `${type} (≤ ${schema.maximum})`;
      }
      if (typeof type !== "string")
        throw new Error("Missing configuration option type");
      return type;
    })
    .join(" or ");
}

function configDefault(schema) {
  if (schema["x-default-description"]) return schema["x-default-description"];
  if (!Object.hasOwn(schema, "default")) return null;
  const value = JSON.stringify(schema.default);
  const longestTick = Math.max(
    0,
    ...(value.match(/`+/g) ?? []).map((s) => s.length),
  );
  const delimiter = "`".repeat(longestTick + 1);
  return `${delimiter}${value}${delimiter}`;
}

export function buildConfigurationMigration(schema) {
  const options = Object.entries(schema.properties).map(([name, option]) => ({
    name,
    type: schemaType(option),
    default_md: configDefault(option) ?? null,
    description_md: option.description,
  }));

  return `CREATE TABLE config_option (
    name TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    default_md TEXT,
    description_md TEXT NOT NULL
);

INSERT INTO config_option
SELECT value ->> 'name', value ->> 'type', value ->> 'default_md', value ->> 'description_md'
FROM json_each('${JSON.stringify(options).replaceAll("'", "''")}');

INSERT INTO documentation_fts(title, description, page, fragment)
SELECT name, description_md, '/configuration', name FROM config_option;
`;
}
