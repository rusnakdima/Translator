import 'dart:convert';
import 'dart:io';

/// Maps Tailwind class substrings → semantic variant names.
/// Order matters: check more specific patterns first.
final _variantInferenceRules = [
  // Button variants
  ['gradient', 'from-indigo', 'to-purple'], 'gradient',
  ['rounded-full', 'shadow-lg'], 'floating',
  ['rounded-full'], 'circle',
  ['hover:bg-indigo-900/30', 'hover:bg-indigo-50'], 'ghost',
  ['border border-indigo-600', 'border border-indigo-500'], 'outline',
  ['bg-red-600', 'bg-red-500', 'hover:bg-red-700', 'hover:bg-red-500'], 'danger',
  ['bg-emerald-600', 'bg-emerald-500', 'hover:bg-emerald-700', 'hover:bg-emerald-500'], 'success',
  ['bg-amber-500', 'bg-amber-600', 'hover:bg-amber-600', 'hover:bg-amber-500'], 'warning',

  // Badge variants
  ['bg-indigo-900', 'text-indigo-200', 'bg-indigo-100', 'text-indigo-800'], 'primary',
  ['bg-emerald-900', 'text-emerald-200', 'bg-emerald-100', 'text-emerald-800'], 'success',
  ['bg-red-900', 'text-red-200', 'bg-red-100', 'text-red-800'], 'danger',
  ['bg-amber-900', 'text-amber-200', 'bg-amber-100', 'text-amber-800'], 'warning',
  ['bg-blue-900', 'text-blue-200', 'bg-blue-100', 'text-blue-800'], 'info',
  ['border border-gray-300', 'border border-gray-600'], 'outline',
  ['bg-gradient-to-r from-indigo-600 to-purple-600'], 'gradient',

  // Card variants
  ['shadow-xl', 'rounded-2xl', 'shadow-md'], 'elevated',
  ['border-2 border-indigo', 'border-2 border-indigo-200'], 'outline',
  ['bg-gradient-to-br from-indigo', 'bg-gradient-to-br from-indigo-50'], 'colored',
  ['rounded-xl', 'shadow', 'border border-neutral'], 'flat',
];

String? inferVariant(String classes) {
  if (classes.isEmpty) return null;

  for (final rule in _variantInferenceRules) {
    final patterns = rule[0] as List<String>;
    bool matches = true;
    for (final pattern in patterns) {
      if (!classes.contains(pattern)) {
        matches = false;
        break;
      }
    }
    if (matches) {
      return rule[1] as String;
    }
  }
  return null;
}

Map<String, dynamic> migrateElement(Map<String, dynamic> el) {
  final classes = (el['classes'] as String?) ?? '';

  // Infer variant from classes
  final variant = inferVariant(classes);

  // Build new element with classes removed (or set to empty string for compatibility)
  final newEl = Map<String, dynamic>.from(el);

  // Remove classes (renderer now ignores it)
  newEl.remove('classes');

  // Add variant if inferrable
  if (variant != null) {
    newEl['variant'] = variant;
  }

  // Recurse into children
  if (newEl.containsKey('children') && newEl['children'] is List) {
    newEl['children'] = (newEl['children'] as List)
        .map((child) => migrateElement(child as Map<String, dynamic>))
        .toList();
  }

  return newEl;
}

void main() async {
  final inputFile = File('${Directory.current.path}/schemas/translator.json');
  final outputFile = File('${Directory.current.path}/schemas/translator_v3.json');

  if (!await inputFile.exists()) {
    print('ERROR: translator.json not found at ${inputFile.path}');
    print('Run from: cd Translator && dart schemas/migrate_to_v3.dart');
    exit(1);
  }

  print('Reading ${inputFile.path}...');
  final content = await inputFile.readAsString();
  final schema = jsonDecode(content) as Map<String, dynamic>;

  print('Migrating pages...');
  if (schema.containsKey('pages') && schema['pages'] is List) {
    schema['pages'] = (schema['pages'] as List).map((page) {
      final p = Map<String, dynamic>.from(page as Map<String, dynamic>);
      if (p.containsKey('elements') && p['elements'] is List) {
        p['elements'] = (p['elements'] as List).map((el) {
          return migrateElement(el as Map<String, dynamic>);
        }).toList();
      }
      // Also migrate page-level layout if string
      if (p.containsKey('layout') && p['layout'] is String) {
        // layout is a string like "stacked" — keep as-is for now
      }
      return p;
    }).toList();
  }

  // Update version
  schema['version'] = '3.0.0';

  print('Writing ${outputFile.path}...');
  final encoder = JsonEncoder.withIndent('  ');
  await outputFile.writeAsString(encoder.convert(schema));

  print('Done! Migrated ${schema['pages'].length} pages to translator_v3.json');
  print('Changes:');
  print('  - Removed classes field (renderer now uses variant + flowbite_mapping)');
  print('  - Added variant field where inferrable from old classes');
  print('  - Version bumped to 3.0.0');
}
