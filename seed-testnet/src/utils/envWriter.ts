import fs from "fs";
import os from "os";

export class EnvWriter {
  private envPath;

  constructor(envPath: string) {
    this.envPath = envPath;
  }

  public setEnvValues = (pair: { [key: string]: string }) => {
    const envVars = this.readEnvVars();
    Object.keys(pair).forEach((key) => {
      this.findAndSubstituteInEnv(key, pair[key], envVars);
    });
    // write everything back to the file system
    fs.writeFileSync(this.envPath, envVars.join(os.EOL));
  };

  //reads lines
  private readEnvVars = () =>
    fs.readFileSync(this.envPath, "utf-8").split(os.EOL);


  private findAndSubstituteInEnv = (
    key: string,
    value: string,
    envVars: string[]
  ) => {
    const targetLine = envVars.find((line) => line.split("=")[0] === key);
    if (targetLine !== undefined) {
      // update existing line
      const targetLineIndex = envVars.indexOf(targetLine);
      // replace the key/value with the new value
      envVars.splice(targetLineIndex, 1, `${key}='${value}'`);
    } else {
      // create new key value
      envVars.push(`${key}='${value}'`);
    }
  };
}
