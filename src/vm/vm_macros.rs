macro_rules! operational_binary_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a).ok_or(SharkyInterrupt::InvalidParameter)?;
                let param_b = $self.read_parameter($b).ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = $self.memory.write();
                let opstack = memory.get_operational_stack();
                let index_a = opstack.read(param_a).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let index_b = opstack.read(param_b).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a $op b)}
                    (_,_) => {return Err(SharkyInterrupt::TypeMismatch);}
                };
                opstack.push(result);
    };
    ($self:ident, $a:expr, $b:expr, $op:tt, real) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a).ok_or(SharkyInterrupt::InvalidParameter)?;
                let param_b = $self.read_parameter($b).ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = $self.memory.write();
                let opstack = memory.get_operational_stack();
                let index_a = opstack.read(param_a).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let index_b = opstack.read(param_b).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a $op b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Real(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a $op b)}
                    (_,_) => {return Err(SharkyInterrupt::TypeMismatch);}
                };
                opstack.push(result);
    };
    // TODO: add b being zero variant. that raises an interrupt
}

macro_rules! operational_unary_impl {

    ($self:ident, $a:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a).ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = $self.memory.write();
                let  opstack = memory.get_operational_stack();
                let index = opstack.read(param_a).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let result = match index {
                    SharkyDataType::Int(a) => {SharkyDataType::Int($op a)}
                    SharkyDataType::Max(a) => {SharkyDataType::Max($op a)}
                    SharkyDataType::Byte(a) => {SharkyDataType::Byte($op a)}
                    _ => {return Err(SharkyInterrupt::TypeMismatch);}
                };
                opstack.push(result);
    };
}

macro_rules! operational_binary_boolean_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a).ok_or(SharkyInterrupt::InvalidParameter)?;
                let param_b = $self.read_parameter($b).ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = $self.memory.write();
                let opstack = memory.get_operational_stack();
                let index_a = opstack.read(param_a).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let index_b = opstack.read(param_b).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let result = match (index_a, index_b) {
                    (SharkyDataType::Bool(a), SharkyDataType::Bool(b)) => {SharkyDataType::Bool(*a $op *b)}
                    (_,_) => {return Err(SharkyInterrupt::TypeMismatch);}
                };
                opstack.push(result);
    };
}

macro_rules! operational_binary_comparison_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a).ok_or(SharkyInterrupt::InvalidParameter)?;
                let param_b = $self.read_parameter($b).ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = $self.memory.write();
                let opstack = memory.get_operational_stack();
                let index_a = opstack.read(param_a).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let index_b = opstack.read(param_b).ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Bool(a), SharkyDataType::Bool(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::HeapReference(a), SharkyDataType::HeapReference(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Nil, SharkyDataType::Nil) => {SharkyDataType::Bool(SharkyDataType::Nil $op SharkyDataType::Nil)}
                    (_,_) => {return Err(SharkyInterrupt::TypeMismatch);} // TODO: return type mismatch interrupt
                };
                opstack.push(result);
    };
}

macro_rules! push_constant {
    ($self:ident, $val:expr, $data_type:ident) => {{
        let parameter = $self
            .read_parameter($val)
            .ok_or(SharkyInterrupt::InvalidParameter)?;
        $self
            .push_constant(SharkyDataType::$data_type(parameter))
            .ok_or(SharkyInterrupt::InvalidStackIndex)?;
    }};
}

macro_rules! convert_match_impl {
    ($self:ident, $a:expr, $stack:ident, $($pattern:pat => $body:expr),* $(,)?) => {
        let param_a = $self.read_parameter($a)
        .ok_or(SharkyInterrupt::InvalidParameter)?;
        let mut memory = $self.memory.write();
        let $stack = memory.get_active_stack_mut()
        .ok_or(SharkyInterrupt::InvalidStackIndex)?;
        let data = $stack.read(param_a)
        .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
        match data {
            $($pattern => $body,)*
            _ => {return Err(SharkyInterrupt::InvalidConversion);}
        }
    }
}
