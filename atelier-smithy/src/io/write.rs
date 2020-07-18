use atelier_core::error::Result;
use atelier_core::io::ModelWriter;
use atelier_core::model::shapes::{AppliedTrait, Member, ShapeKind};
use atelier_core::model::values::Value as NodeValue;
use atelier_core::model::Model;
use atelier_core::syntax::{
    MEMBER_COLLECTION_OPERATIONS, MEMBER_CREATE, MEMBER_DELETE, MEMBER_ERRORS, MEMBER_IDENTIFIERS,
    MEMBER_INPUT, MEMBER_KEY, MEMBER_LIST, MEMBER_MEMBER, MEMBER_OPERATIONS, MEMBER_OUTPUT,
    MEMBER_PUT, MEMBER_READ, MEMBER_RESOURCES, MEMBER_UPDATE, MEMBER_VALUE, MEMBER_VERSION,
    SHAPE_APPLY, SHAPE_LIST, SHAPE_MAP, SHAPE_OPERATION, SHAPE_RESOURCE, SHAPE_SERVICE, SHAPE_SET,
    SHAPE_STRUCTURE, SHAPE_UNION,
};
use std::io::Write;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// Write a [Model](../atelier_core/model/struct.Model.html) in the Smithy native representation.
///
#[derive(Debug)]
pub struct SmithyWriter;

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

const CONTROL_DATA_PREFIX: &str = "$";
const TRAIT_PREFIX: &str = "@";

const STATEMENT_NAMESPACE: &str = "namespace";
const STATEMENT_USE: &str = "use";
const STATEMENT_METADATA: &str = "metadata";

impl<'a> Default for SmithyWriter {
    fn default() -> Self {
        Self {}
    }
}

impl<'a> ModelWriter<'a> for SmithyWriter {
    fn write(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        self.write_control_section(w, model)?;
        self.write_metadata_section(w, model)?;
        self.write_shape_section(w, model)
    }
}

impl<'a> SmithyWriter {
    fn write_control_section(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        writeln!(
            w,
            "${}: \"{}\"",
            MEMBER_VERSION,
            model.version().to_string()
        )?;
        writeln!(w)?;
        if model.has_control_data() {
            for (key, value) in model.control_data() {
                writeln!(w, "{}{} = {}", CONTROL_DATA_PREFIX, key, value)?;
            }
            writeln!(w)?;
        }
        Ok(())
    }

    fn write_metadata_section(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        if model.has_metadata() {
            for (key, value) in model.metadata() {
                writeln!(w, "{} \"{}\" = {}", STATEMENT_METADATA, key, value)?;
            }
            writeln!(w)?;
        }
        Ok(())
    }

    fn write_shape_section(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        self.write_namespace_statement(w, model)?;
        self.write_use_section(w, model)?;
        self.write_shape_statements(w, model)
    }
    fn write_namespace_statement(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        writeln!(w, "{} {}", STATEMENT_NAMESPACE, model.namespace())?;
        writeln!(w)?;
        Ok(())
    }

    fn write_use_section(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        for use_shape in model.references() {
            writeln!(w, "{} {}", STATEMENT_USE, use_shape)?;
        }
        writeln!(w)?;
        Ok(())
    }

    fn write_shape_statements(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        for shape in model.shapes() {
            if !shape.body().is_apply() {
                for a_trait in shape.traits() {
                    self.write_trait(w, a_trait, "")?;
                }
            }
            match shape.body() {
                ShapeKind::SimpleType(st) => {
                    writeln!(w, "{} {}", st, shape.id())?;
                }
                ShapeKind::List(list) => {
                    writeln!(w, "{} {} {{", SHAPE_LIST, shape.id())?;
                    writeln!(w, "    {}: {}", MEMBER_MEMBER, list.member())?;
                    writeln!(w, "}}")?;
                }
                ShapeKind::Set(set) => {
                    writeln!(w, "{} {} {{", SHAPE_SET, shape.id())?;
                    writeln!(w, "    {}: {}", MEMBER_MEMBER, set.member())?;
                    writeln!(w, "}}")?;
                }
                ShapeKind::Map(map) => {
                    writeln!(w, "{} {} {{", SHAPE_MAP, shape.id())?;
                    writeln!(w, "    {}: {}", MEMBER_KEY, map.key())?;
                    writeln!(w, "    {}: {}", MEMBER_VALUE, map.value())?;
                    writeln!(w, "}}")?;
                }
                ShapeKind::Structure(structured) => {
                    writeln!(w, "{} {} {{", SHAPE_STRUCTURE, shape.id())?;
                    self.write_members(w, structured.members(), "    ")?;
                    writeln!(w, "}}")?;
                }
                ShapeKind::Union(structured) => {
                    writeln!(w, "{} {} {{", SHAPE_UNION, shape.id())?;
                    self.write_members(w, structured.members(), "    ")?;
                    writeln!(w, "}}")?;
                }
                ShapeKind::Service(service) => {
                    writeln!(w, "{} {} {{", SHAPE_SERVICE, shape.id())?;
                    writeln!(w, "    {}: {}", MEMBER_VERSION, service.version())?;
                    if service.has_operations() {
                        writeln!(
                            w,
                            "    {}: [{}]",
                            MEMBER_OPERATIONS,
                            service
                                .operations()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )?;
                    }
                    if service.has_resources() {
                        writeln!(
                            w,
                            "    {}: [{}]",
                            MEMBER_RESOURCES,
                            service
                                .resources()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )?;
                    }
                    writeln!(w, "}}")?;
                }
                ShapeKind::Operation(operation) => {
                    writeln!(w, "{} {} {{", SHAPE_OPERATION, shape.id())?;
                    if let Some(id) = operation.input() {
                        writeln!(w, "    {}: {}", MEMBER_INPUT, id)?;
                    }
                    if let Some(id) = operation.output() {
                        writeln!(w, "    {}: {}", MEMBER_OUTPUT, id)?;
                    }
                    if operation.has_errors() {
                        writeln!(
                            w,
                            "    {}: [{}]",
                            MEMBER_ERRORS,
                            operation
                                .errors()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )?;
                    }
                    writeln!(w, "}}")?;
                }
                ShapeKind::Resource(resource) => {
                    writeln!(w, "{} {} {{", SHAPE_RESOURCE, shape.id())?;
                    if resource.has_identifiers() {
                        writeln!(w, "    {}: {{", MEMBER_IDENTIFIERS)?;
                        for (id, ref_id) in resource.identifiers() {
                            writeln!(w, "        {}: {}", id, ref_id)?;
                        }
                        writeln!(w, "    }}")?;
                    }
                    if let Some(id) = resource.create() {
                        writeln!(w, "    {}: {}", MEMBER_CREATE, id)?;
                    }
                    if let Some(id) = resource.put() {
                        writeln!(w, "    {}: {}", MEMBER_PUT, id)?;
                    }
                    if let Some(id) = resource.read() {
                        writeln!(w, "    {}: {}", MEMBER_READ, id)?;
                    }
                    if let Some(id) = resource.update() {
                        writeln!(w, "    {}: {}", MEMBER_UPDATE, id)?;
                    }
                    if let Some(id) = resource.delete() {
                        writeln!(w, "    {}: {}", MEMBER_DELETE, id)?;
                    }
                    if let Some(id) = resource.list() {
                        writeln!(w, "    {}: {}", MEMBER_LIST, id)?;
                    }
                    if resource.has_operations() {
                        writeln!(
                            w,
                            "    {}: [{}]",
                            MEMBER_OPERATIONS,
                            resource
                                .operations()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )?;
                    }
                    if resource.has_collection_operations() {
                        writeln!(
                            w,
                            "    {}: [{}]",
                            MEMBER_COLLECTION_OPERATIONS,
                            resource
                                .collection_operations()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )?;
                    }
                    if resource.has_resources() {
                        writeln!(
                            w,
                            "    {}: [{}]",
                            MEMBER_RESOURCES,
                            resource
                                .resources()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )?;
                    }
                    writeln!(w, "}}")?;
                }
                ShapeKind::Apply => {
                    for a_trait in shape.traits() {
                        write!(w, "{} {} ", SHAPE_APPLY, shape.id())?;
                        self.write_trait(w, a_trait, "")?;
                    }
                }
            }
            writeln!(w)?;
        }
        Ok(())
    }

    fn write_trait(&mut self, w: &mut impl Write, a_trait: &'a Trait, prefix: &str) -> Result<()> {
        write!(w, "{}{}{}", prefix, TRAIT_PREFIX, a_trait.id())?;
        match a_trait.value() {
            None => writeln!(w)?,
            Some(NodeValue::Object(map)) => writeln!(
                w,
                "({})",
                map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?,
            Some(value) => writeln!(w, "({})", value.to_string())?,
        }
        Ok(())
    }

    fn write_members(
        &mut self,
        w: &mut impl Write,
        members: impl Iterator<Item = &'a Member>,
        prefix: &str,
    ) -> Result<()> {
        for member in members {
            self.write_member(w, member, prefix)?;
        }
        Ok(())
    }

    fn write_member(&mut self, w: &mut impl Write, member: &'a Member, prefix: &str) -> Result<()> {
        for a_trait in member.traits() {
            self.write_trait(w, a_trait, prefix)?;
        }
        writeln!(
            w,
            "{}{}: {}",
            prefix,
            member.id(),
            member.value().as_ref().unwrap()
        )?;
        Ok(())
    }
}

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------
