use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;

pub trait PBOSigner {
    type PBOSignError;

    fn sign<A: AsRef<Path>, B: AsRef<Path>, C: AsRef<Path>>(&self, pbo_path: A, private_key_path: B, output_path: C) -> Result<(), Self::PBOSignError>;
}

pub struct ArmaToolsPBOSigner<P: AsRef<Path>> {
    pub tool_path: P
}

impl<P: AsRef<Path>> PBOSigner for ArmaToolsPBOSigner<P>
{
    type PBOSignError = Error;

    fn sign<A: AsRef<Path>, B: AsRef<Path>, C: AsRef<Path>>(&self, pbo_path: A, private_key_path: B, output_path: C) -> Result<(), Self::PBOSignError> {
        let pbo_path = pbo_path.as_ref();
        let private_key_path = private_key_path.as_ref();
        let output_path = output_path.as_ref();

        let output = Command::new(self.tool_path.as_ref())
            .current_dir(output_path)
            .arg(private_key_path.as_os_str())
            .arg(pbo_path.as_os_str())
            .output()?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(0);

            let stdout = String::from_utf8(output.stdout);
            let stderr = String::from_utf8(output.stderr);

            if let Ok(stdout) = stdout {
                println!("{}", stdout);
            }

            if let Ok(stderr) = stderr {
                eprintln!("{}", stderr);
            }

            let output = format!("PBO Signing failed, error code: {code}");

            return Err(Error::new(ErrorKind::Other, output));
        }

        Ok(())
    }
}