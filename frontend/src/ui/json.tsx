type JsonValue =
  | string
  | number
  | boolean
  | null
  | undefined
  | JsonValue[]
  | { [key: string]: JsonValue };

interface JsonProps {
  json: JsonValue;
}

export const Json = ({ json }: JsonProps) => {
  if (!json) {
    return <p>null</p>;
  }

  const type = typeof json;

  if (type === "function") {
    return <p>??function??</p>;
  }

  // null case
  if (type === "undefined") {
    return <p>null</p>;
  }

  // base cases
  if (
    type === "bigint" ||
    type === "boolean" ||
    type === "number" ||
    type === "string" ||
    type === "symbol"
  ) {
    return <p>{String(json)}</p>;
  }

  // Type is object or array
  if (Array.isArray(json)) {
    return (
      <div className="flex flex-col gap-2">
        {json.map((item, index) => (
          <Json key={index} json={item} />
        ))}
      </div>
    );
  }

  // Type is object
  if (type === "object" && json !== null) {
    return (
      <div className="flex flex-col gap-2">
        {Object.keys(json).map((key) => (
          <div key={key} className="flex gap-2">
            <p>{key}</p>: <Json json={(json as Record<string, JsonValue>)[key]} />
          </div>
        ))}
      </div>
    );
  }

  return <p>null</p>;
};
