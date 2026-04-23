import toJsonSchema from 'to-json-schema';

export function inferJsonSchema(data: any): any {
  if (data === null || data === undefined) return { type: 'null' };
  
  return toJsonSchema(data, {
    required: false,
    objects: {
      additionalProperties: false
    },
    arrays: {
      mode: 'first' // 'first' is usually better for API docs than 'all' which can get messy
    }
  });
}

export function inferTypeScript(data: any, interfaceName: string = 'Root'): string {
  const schema = inferJsonSchema(data);
  const lines: string[] = [];
  const processedObjects = new Map<string, string>();

  function generateInterface(obj: any, name: string): string {
    if (obj.type !== 'object' || !obj.properties) {
      return getTsType(obj);
    }

    const cacheKey = JSON.stringify(obj.properties);
    if (processedObjects.has(cacheKey)) {
      return processedObjects.get(cacheKey)!;
    }

    const safeName = name.charAt(0).toUpperCase() + name.slice(1);
    processedObjects.set(cacheKey, safeName);

    let interfaceStr = `export interface ${safeName} {\n`;
    for (const [prop, details] of Object.entries(obj.properties as any)) {
      const typeStr = getTsType(details, prop);
      interfaceStr += `  ${prop}: ${typeStr};\n`;
    }
    interfaceStr += `}\n`;
    
    lines.push(interfaceStr);
    return safeName;
  }

  function getTsType(details: any, name: string = ''): string {
    switch (details.type) {
      case 'string': return 'string';
      case 'number':
      case 'integer': return 'number';
      case 'boolean': return 'boolean';
      case 'null': return 'null';
      case 'array':
        const itemType = details.items ? getTsType(details.items, name + 'Item') : 'any';
        return `${itemType}[]`;
      case 'object':
        return generateInterface(details, name || 'SubObject');
      default:
        return 'any';
    }
  }

  const rootType = getTsType(schema, interfaceName);
  
  // If the root was an object, the interface is already in 'lines'
  // If the root was a primitive or array, we add a type alias
  if (schema.type !== 'object') {
    return `export type ${interfaceName} = ${rootType};\n\n` + lines.join('\n');
  }

  return lines.reverse().join('\n');
}
